package handlers

import (
	"encoding/json"
	"net/http"
	"strconv"
	"sync"
	"time"

	"github.com/cyber-kube/api-go/internal/database"
	"github.com/cyber-kube/api-go/internal/services"
	"github.com/cyber-kube/api-go/pkg/auth"
	"github.com/gin-gonic/gin"
	"github.com/gorilla/websocket"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

type Handlers struct {
	k8sService    *services.K8sService
	policyService *services.PolicyService
	alertService  *services.AlertService
	eventsHub     *eventsHub
}

func New(k8s *services.K8sService, policy *services.PolicyService, alert *services.AlertService) *Handlers {
	hub := newEventsHub()
	go hub.run()

	return &Handlers{
		k8sService:    k8s,
		policyService: policy,
		alertService:  alert,
		eventsHub:     hub,
	}
}

const (
	maxEventConnections = 100
	eventPingInterval   = 30 * time.Second
	eventPongWait       = 60 * time.Second
	eventWriteWait      = 10 * time.Second
)

var (
	httpRequestsTotal = prometheus.NewCounterVec(
		prometheus.CounterOpts{
			Name: "cyberkube_http_requests_total",
			Help: "Total HTTP requests by endpoint and status.",
		},
		[]string{"endpoint", "status"},
	)
	activeWebSocketConnections = prometheus.NewGauge(
		prometheus.GaugeOpts{
			Name: "cyberkube_websocket_connections_active",
			Help: "Current number of active WebSocket connections.",
		},
	)
	alertsBySeverity = prometheus.NewGaugeVec(
		prometheus.GaugeOpts{
			Name: "cyberkube_alerts_by_severity",
			Help: "Current number of alerts grouped by severity.",
		},
		[]string{"severity"},
	)
	k8sPodsByNamespace = prometheus.NewGaugeVec(
		prometheus.GaugeOpts{
			Name: "cyberkube_k8s_pods_by_namespace",
			Help: "Current number of Kubernetes pods grouped by namespace.",
		},
		[]string{"namespace"},
	)
)

func init() {
	prometheus.MustRegister(
		httpRequestsTotal,
		activeWebSocketConnections,
		alertsBySeverity,
		k8sPodsByNamespace,
	)
}

var eventsUpgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin: func(r *http.Request) bool {
		return true
	},
}

type eventsHub struct {
	clients    map[*eventClient]struct{}
	register   chan *eventClient
	unregister chan *eventClient
	broadcast  chan []byte
	mu         sync.RWMutex
}

type eventClient struct {
	hub  *eventsHub
	conn *websocket.Conn
	send chan []byte
}

func newEventsHub() *eventsHub {
	return &eventsHub{
		clients:    make(map[*eventClient]struct{}),
		register:   make(chan *eventClient),
		unregister: make(chan *eventClient),
		broadcast:  make(chan []byte, 256),
	}
}

func (h *eventsHub) run() {
	for {
		select {
		case client := <-h.register:
			h.mu.Lock()
			h.clients[client] = struct{}{}
			activeWebSocketConnections.Set(float64(len(h.clients)))
			h.mu.Unlock()
		case client := <-h.unregister:
			h.mu.Lock()
			if _, ok := h.clients[client]; ok {
				delete(h.clients, client)
				close(client.send)
			}
			activeWebSocketConnections.Set(float64(len(h.clients)))
			h.mu.Unlock()
		case message := <-h.broadcast:
			h.mu.RLock()
			clients := make([]*eventClient, 0, len(h.clients))
			for client := range h.clients {
				clients = append(clients, client)
			}
			h.mu.RUnlock()

			for _, client := range clients {
				select {
				case client.send <- message:
				default:
					h.unregister <- client
				}
			}
		}
	}
}

func observeRequest(c *gin.Context) {
	endpoint := c.FullPath()
	if endpoint == "" {
		endpoint = c.Request.URL.Path
	}
	httpRequestsTotal.WithLabelValues(endpoint, strconv.Itoa(c.Writer.Status())).Inc()
}

func (h *Handlers) refreshAlertMetrics() {
	alertsBySeverity.Reset()

	for _, alert := range h.alertService.List() {
		severity := alert.Severity
		if severity == "" {
			severity = "unknown"
		}
		alertsBySeverity.WithLabelValues(severity).Inc()
	}
}

func (h *Handlers) refreshK8sPodMetrics() {
	k8sPodsByNamespace.Reset()

	if h.k8sService == nil {
		return
	}

	namespaces, err := h.k8sService.ListNamespaces()
	if err != nil {
		return
	}

	for _, namespace := range namespaces {
		pods, err := h.k8sService.ListPods(namespace.Name)
		if err != nil {
			continue
		}
		k8sPodsByNamespace.WithLabelValues(namespace.Name).Set(float64(len(pods)))
	}
}

func (h *eventsHub) clientCount() int {
	h.mu.RLock()
	defer h.mu.RUnlock()
	return len(h.clients)
}

func (h *eventsHub) broadcastJSON(messageType string, payload interface{}) {
	message, err := json.Marshal(gin.H{
		"type": messageType,
		"data": payload,
	})
	if err != nil {
		return
	}

	select {
	case h.broadcast <- message:
	default:
	}
}

func (c *eventClient) readPump() {
	defer func() {
		c.hub.unregister <- c
		_ = c.conn.Close()
	}()

	c.conn.SetReadLimit(1024)
	_ = c.conn.SetReadDeadline(time.Now().Add(eventPongWait))
	c.conn.SetPongHandler(func(string) error {
		return c.conn.SetReadDeadline(time.Now().Add(eventPongWait))
	})

	for {
		if _, _, err := c.conn.ReadMessage(); err != nil {
			return
		}
	}
}

func (c *eventClient) writePump() {
	ticker := time.NewTicker(eventPingInterval)
	defer func() {
		ticker.Stop()
		_ = c.conn.Close()
	}()

	for {
		select {
		case message, ok := <-c.send:
			_ = c.conn.SetWriteDeadline(time.Now().Add(eventWriteWait))
			if !ok {
				_ = c.conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}
			if err := c.conn.WriteMessage(websocket.TextMessage, message); err != nil {
				return
			}
		case <-ticker.C:
			_ = c.conn.SetWriteDeadline(time.Now().Add(eventWriteWait))
			if err := c.conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}

// Health check handler (reuse existing from main.go)
func (h *Handlers) Health(c *gin.Context) {
	defer observeRequest(c)
	c.JSON(200, gin.H{"status": "ok"})
}

// Policy handlers (delegate to existing handlers.go functions)
func (h *Handlers) CreatePolicy(c *gin.Context) {
	defer observeRequest(c)
	var policy database.Policy
	if err := c.ShouldBindJSON(&policy); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	created, err := h.policyService.Create(policy)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	h.eventsHub.broadcastJSON("policy", created)
	c.JSON(http.StatusCreated, created)
}

func (h *Handlers) ListPolicies(c *gin.Context) {
	defer observeRequest(c)
	policies := h.policyService.List()
	c.JSON(200, policies)
}

func (h *Handlers) GetPolicy(c *gin.Context) {
	defer observeRequest(c)
	id := c.Param("id")
	policy := h.policyService.Get(id)
	if policy == nil {
		c.JSON(404, gin.H{"error": "not found"})
		return
	}
	c.JSON(200, policy)
}

func (h *Handlers) UpdatePolicy(c *gin.Context) {
	defer observeRequest(c)
	id := c.Param("id")
	// Update logic
	c.JSON(200, gin.H{"id": id, "message": "updated"})
}

func (h *Handlers) DeletePolicy(c *gin.Context) {
	defer observeRequest(c)
	id := c.Param("id")
	h.policyService.Delete(id)
	c.JSON(204, nil)
}

// Alert handlers
func (h *Handlers) ListAlerts(c *gin.Context) {
	defer observeRequest(c)
	alerts := h.alertService.List()
	c.JSON(200, alerts)
}

func (h *Handlers) CreateAlert(c *gin.Context) {
	defer observeRequest(c)
	var alert database.Alert
	if err := c.ShouldBindJSON(&alert); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	created, err := h.alertService.Create(alert)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	h.eventsHub.broadcastJSON("alert", created)
	c.JSON(http.StatusCreated, created)
}

func (h *Handlers) GetAlert(c *gin.Context) {
	defer observeRequest(c)
	id := c.Param("id")
	alert := h.alertService.Get(id)
	if alert == nil {
		c.JSON(404, gin.H{"error": "not found"})
		return
	}
	c.JSON(200, alert)
}

// WebSocket events handler
func (h *Handlers) EventsWS(c *gin.Context) {
	defer observeRequest(c)
	token := c.Query("token")
	if token == "" {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "token query parameter required"})
		return
	}

	claims, err := auth.ValidateToken(token)
	if err != nil {
		c.JSON(http.StatusUnauthorized, gin.H{"error": "invalid token"})
		return
	}

	if h.eventsHub.clientCount() >= maxEventConnections {
		c.JSON(http.StatusServiceUnavailable, gin.H{"error": "maximum websocket connections reached"})
		return
	}

	conn, err := eventsUpgrader.Upgrade(c.Writer, c.Request, nil)
	if err != nil {
		return
	}

	client := &eventClient{
		hub:  h.eventsHub,
		conn: conn,
		send: make(chan []byte, 16),
	}

	c.Set("username", claims.Username)
	h.eventsHub.register <- client
	h.eventsHub.broadcastJSON("connection", gin.H{"username": claims.Username, "status": "connected"})

	go client.writePump()
	client.readPump()
}

// Metrics handler
func (h *Handlers) Metrics(c *gin.Context) {
	defer observeRequest(c)
	h.refreshAlertMetrics()
	h.refreshK8sPodMetrics()
	promhttp.Handler().ServeHTTP(c.Writer, c.Request)
}
