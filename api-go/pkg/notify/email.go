package notify

import (
	"fmt"
	"net/smtp"

	"github.com/cyber-kube/api-go/internal/config"
	"github.com/cyber-kube/api-go/internal/database"
)

type AlertPayload struct {
	ID        string
	Kind      string
	Severity  string
	Message   string
	Source    string
	Namespace string
}

func SendAlertEmail(cfg *config.Config, settings database.AppSettings, alert AlertPayload) error {
	if cfg == nil {
		return fmt.Errorf("notification config is unavailable")
	}
	if !settings.EmailNotifications {
		return nil
	}
	if cfg.SMTPHost == "" || cfg.SMTPFrom == "" {
		return fmt.Errorf("smtp host/from configuration is incomplete")
	}

	to := settings.NotificationEmail
	if to == "" {
		to = cfg.SMTPTo
	}
	if to == "" {
		return fmt.Errorf("no notification recipient configured")
	}

	subject := fmt.Sprintf("Cyber-Kube alert: %s (%s)", alert.Kind, alert.Severity)
	body := fmt.Sprintf(
		"Alert ID: %s\r\nKind: %s\r\nSeverity: %s\r\nMessage: %s\r\nSource: %s\r\nNamespace: %s\r\n",
		alert.ID,
		alert.Kind,
		alert.Severity,
		alert.Message,
		alert.Source,
		alert.Namespace,
	)
	return SendEmail(cfg, to, subject, body)
}

func SendEmail(cfg *config.Config, to string, subject string, body string) error {
	if cfg == nil {
		return fmt.Errorf("notification config is unavailable")
	}
	if cfg.SMTPHost == "" || cfg.SMTPFrom == "" {
		return fmt.Errorf("smtp host/from configuration is incomplete")
	}
	if to == "" {
		to = cfg.SMTPTo
	}
	if to == "" {
		return fmt.Errorf("no notification recipient configured")
	}

	address := fmt.Sprintf("%s:%s", cfg.SMTPHost, cfg.SMTPPort)
	message := []byte(fmt.Sprintf("To: %s\r\nSubject: %s\r\n\r\n%s", to, subject, body))

	if cfg.SMTPUsername == "" && cfg.SMTPPassword == "" {
		return smtp.SendMail(address, nil, cfg.SMTPFrom, []string{to}, message)
	}

	auth := smtp.PlainAuth("", cfg.SMTPUsername, cfg.SMTPPassword, cfg.SMTPHost)
	return smtp.SendMail(address, auth, cfg.SMTPFrom, []string{to}, message)
}
