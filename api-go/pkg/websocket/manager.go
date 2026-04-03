package websocket

import (
	"encoding/json"
	"log"
	"net/http"
	"time"

	"github.com/cyber-kube/api-go/internal/models"
	"github.com/gorilla/websocket"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		return true // Allow all origins in dev
	},
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
}

type Manager struct {
	clients    map[*websocket.Conn]bool
	broadcast  chan []byte
	register   chan *websocket.Conn
	unregister chan *websocket.Conn
}

func NewManager() *Manager {
	return &Manager{
		clients:    make(map[*websocket.Conn]bool),
		broadcast:  make(chan []byte, 256),
		register:   make(chan *websocket.Conn),
		unregister: make(chan *websocket.Conn),
	}
}

func (m *Manager) Start() {
	for {
		select {
		case conn := <-m.register:
			m.clients[conn] = true
			log.Printf("WebSocket client connected. Total clients: %d", len(m.clients))

		case conn := <-m.unregister:
			if _, ok := m.clients[conn]; ok {
				delete(m.clients, conn)
				conn.Close()
				log.Printf("WebSocket client disconnected. Total clients: %d", len(m.clients))
			}

		case message := <-m.broadcast:
			for client := range m.clients {
				client.SetWriteDeadline(time.Now().Add(10 * time.Second))
				if err := client.WriteMessage(websocket.TextMessage, message); err != nil {
					delete(m.clients, client)
					client.Close()
				}
			}
		}
	}
}

func (m *Manager) HandleConnection(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("WebSocket upgrade error: %v", err)
		return
	}

	m.register <- conn

	// Keep connection alive and handle pings
	go func() {
		defer func() {
			m.unregister <- conn
		}()

		conn.SetReadDeadline(time.Now().Add(60 * time.Second))
		conn.SetPongHandler(func(string) error {
			conn.SetReadDeadline(time.Now().Add(60 * time.Second))
			return nil
		})

		ticker := time.NewTicker(54 * time.Second)
		defer ticker.Stop()

		for {
			select {
			case <-ticker.C:
				if err := conn.WriteMessage(websocket.PingMessage, nil); err != nil {
					return
				}
			default:
				_, _, err := conn.ReadMessage()
				if err != nil {
					if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
						log.Printf("WebSocket error: %v", err)
					}
					return
				}
			}
		}
	}()
}

func (m *Manager) Broadcast(message []byte) {
	select {
	case m.broadcast <- message:
	default:
		log.Println("Broadcast channel full, dropping message")
	}
}

func (m *Manager) BroadcastAlert(alert models.Alert) {
	message := map[string]interface{}{
		"type": "alert",
		"data": alert,
	}
	data, err := json.Marshal(message)
	if err != nil {
		log.Printf("Error marshaling alert: %v", err)
		return
	}
	m.Broadcast(data)
}

func (m *Manager) BroadcastEvent(event models.Event) {
	message := map[string]interface{}{
		"type": "event",
		"data": event,
	}
	data, err := json.Marshal(message)
	if err != nil {
		log.Printf("Error marshaling event: %v", err)
		return
	}
	m.Broadcast(data)
}
