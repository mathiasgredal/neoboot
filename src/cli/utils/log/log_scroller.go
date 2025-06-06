package log

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"strings"
	"sync"
	"time"

	"github.com/sirupsen/logrus"
)

const (
	// ANSI Escape Codes
	hideCursor      = "\033[?25l"
	showCursor      = "\033[?25h"
	disableLineWrap = "\033[?7l"
	enableLineWrap  = "\033[?7h"
	clearLine       = "\033[K"
	cursorUp        = "\033[A"
	colorResetCode  = "\033[0m"
)

// LogScroller manages the animation and display.
type LogScroller struct {
	writer         io.Writer
	title          string
	spinnerChars   []string
	spinnerCursor  int
	animationSpeed time.Duration
	linesToDisplay int      // Max number of log_scroller's own log lines to display
	logBuffer      []string // Holds the log_scrollers's own log lines
	logChan        chan string
	stopChan       chan struct{}
	wg             sync.WaitGroup
	mu             sync.Mutex
	active         bool
	finalStatus    string

	enableColor  bool
	colorSpinner string
	colorTitle   string
	colorLogs    string
	colorSuccess string
	colorError   string
	colorFinish  string

	charSuccess string
	charError   string
	charFinish  string

	linesPrinted int // Number of lines written in the last draw by the log scroller

	triggerRedrawSignal chan struct{}
}

// Option defines a function type for configuring the LogScroller.
type Option func(*LogScroller)

// NotifyingWriter wraps an io.Writer and signals the log scroller for an immediate redraw after a write.
type NotifyingWriter struct {
	originalWriter io.Writer
	logScroller    *LogScroller
}

// SetupLogging configures logrus to use the log scroller for logging.
func setupLogging(log *logrus.Logger, s *LogScroller) {
	log.SetOutput(NewNotifyingWriter(os.Stderr, s))
	log.SetFormatter(&LogrusLogScrollerFormatter{
		LogScroller:     s,
		TimestampFormat: time.RFC3339,
	})
}

// TeardownLogging restores the standard logrus settings.
func teardownLogging(log *logrus.Logger) {
	log.SetOutput(NewNotifyingWriter(os.Stderr, nil))
	log.SetFormatter(&LogrusLogScrollerFormatter{
		LogScroller:     nil,
		TimestampFormat: time.RFC3339,
	})
}

// NewNotifyingWriter creates a writer that signals the log scroller for an immediate redraw.
func NewNotifyingWriter(original io.Writer, s *LogScroller) *NotifyingWriter {
	return &NotifyingWriter{originalWriter: original, logScroller: s}
}

// Write performs the write and then signals the log scroller for an immediate redraw.
func (nw *NotifyingWriter) Write(p []byte) (n int, err error) {
	n, err = nw.originalWriter.Write(p) // Write the actual log data
	if err != nil {
		return n, err
	}

	// After successful write, try to signal the log scroller for an immediate redraw
	if nw.logScroller != nil {
		nw.logScroller.mu.Lock() // Lock log scroller to safely check active and send signal
		if nw.logScroller.active && nw.logScroller.triggerRedrawSignal != nil {
			select {
			case nw.logScroller.triggerRedrawSignal <- struct{}{}: // Non-blocking send
			default:
				// Channel is full or log scroller is busy; redraw will happen on next tick.
				// This is fine, we don't want to block the log writer.
			}
		}
		nw.logScroller.mu.Unlock()
	}
	return n, err
}

// NewLogScroller creates a new LogScroller instance.
// The writer is typically os.Stderr.
// Title is the text displayed next to the log scroller.
func NewLogScroller(writer io.Writer, title string, options ...Option) *LogScroller {
	s := &LogScroller{
		writer:         writer,
		title:          title,
		spinnerChars:   []string{"⣷", "⣯", "⣟", "⡿", "⢿", "⣻", "⣽", "⣾"},
		animationSpeed: 50 * time.Millisecond, // Matches _SPIN_STREAM_SLEEP
		linesToDisplay: 5,
		logChan:        make(chan string, 100),
		stopChan:       make(chan struct{}),
		active:         false,
		enableColor:    true, // Colors enabled by default
		// Default colors matching the bash script's scheme where appropriate
		colorSpinner:        "\033[38;5;209m", // SPIN_COLOR_SPINNER (Orange-ish)
		colorTitle:          "\033[38;5;69m",  // SPIN_COLOR_TITLE (Blue-ish)
		colorLogs:           "\033[38;5;240m", // SPIN_COLOR_LOGS (Grey)
		colorSuccess:        "\033[32m",       // Green (standard)
		colorError:          "\033[31m",       // Red (standard)
		colorFinish:         "\033[38;5;209m", // Default finish char color to spinner color
		charSuccess:         "✔",              // SPIN_CHAR_SUCCESS
		charError:           "✘",              // SPIN_CHAR_ERROR
		charFinish:          "⣿",              // SPIN_CHAR_FINISH
		linesPrinted:        0,
		triggerRedrawSignal: make(chan struct{}, 1),
	}

	for _, opt := range options {
		opt(s)
	}
	return s
}

// --- Options ---

// WithLogScrollerChars sets the characters used for the log scroller animation.
func WithLogScrollerChars(chars []string) Option {
	return func(s *LogScroller) {
		if len(chars) > 0 {
			s.spinnerChars = chars
		}
	}
}

// WithAnimationSpeed sets the delay between log scroller frames.
func WithAnimationSpeed(speed time.Duration) Option {
	return func(s *LogScroller) {
		s.animationSpeed = speed
	}
}

// WithLinesToDisplay sets how many log lines are shown.
func WithLinesToDisplay(lines int) Option {
	return func(s *LogScroller) {
		if lines > 0 {
			s.linesToDisplay = lines
		}
	}
}

// WithDisabledColor disables ANSI color output.
func WithDisabledColor() Option {
	return func(s *LogScroller) {
		s.enableColor = false
	}
}

// WithCustomChars sets custom characters for final states.
func WithCustomChars(success, error, finish string) Option {
	return func(s *LogScroller) {
		if success != "" {
			s.charSuccess = success
		}
		if error != "" {
			s.charError = error
		}
		if finish != "" {
			s.charFinish = finish
		}
	}
}

// WithCustomColors sets custom ANSI color codes.
// Provide full ANSI codes, e.g., "\033[31m" for red.
func WithCustomColors(logScroller, title, logs, success, error, finish string) Option {
	return func(s *LogScroller) {
		if logScroller != "" {
			s.colorSpinner = logScroller
		}
		if title != "" {
			s.colorTitle = title
		}
		if logs != "" {
			s.colorLogs = logs
		}
		if success != "" {
			s.colorSuccess = success
		}
		if error != "" {
			s.colorError = error
		}
		if finish != "" {
			s.colorFinish = finish
		}
	}
}

// --- Control Methods ---
func (s *LogScroller) Start() {
	s.mu.Lock()
	if s.active {
		s.mu.Unlock()
		return
	}
	s.active = true
	// Ensure channels are fresh if Start can be called multiple times (or re-init them)
	// For this design, assume NewLogScroller creates them fresh.

	setupLogging(log, s)
	s.mu.Unlock()

	if s.enableColor {
		fmt.Fprint(s.writer, hideCursor)
		fmt.Fprint(s.writer, disableLineWrap)
	}

	s.wg.Add(1)
	go s.animate()
}

func (s *LogScroller) stopInternal(status string) {
	s.mu.Lock()
	if !s.active {
		s.mu.Unlock()
		return
	}
	s.active = false
	s.finalStatus = status
	// Drain triggerRedrawSignal on stop to prevent stale signals if log scroller is restarted.
	// And to allow animate goroutine to exit cleanly if it's blocked on this.
	close(s.stopChan) // Signal animator to stop FIRST
	s.mu.Unlock()     // Unlock before Wait to avoid deadlock if animate needs lock for final draw

	s.wg.Wait() // Wait for animator to finish

	if s.enableColor {
		fmt.Fprint(s.writer, showCursor)
		fmt.Fprint(s.writer, enableLineWrap)
		fmt.Fprint(s.writer, colorResetCode)
	}

	teardownLogging(log)
}

// Stop halts the log scroller, showing a generic finish character.
func (s *LogScroller) Stop() {
	s.stopInternal("")
}

// Success halts the log scroller, showing the success character and message.
func (s *LogScroller) Success() {
	s.stopInternal("success")
}

// Error halts the log scroller, showing the error character and message.
func (s *LogScroller) Error() {
	s.stopInternal("error")
}

// AddLog sends a log message to be displayed beneath the log scroller.
// It can be called multiple times. Each call adds a new line.
// If the message contains newlines, it will be split into multiple log entries.
func (s *LogScroller) AddLog(message string) {
	s.mu.Lock()
	isActive := s.active
	s.mu.Unlock()

	if isActive {
		lines := strings.Split(message, "\n")
		for _, line := range lines {
			s.logChan <- line
		}
	} else {
		// If log scroller is stopped, append to buffer for potential final draw if needed,
		// or just print if no more draws are expected.
		// For simplicity, logs added after stop are currently not displayed by the log scroller.
		// Consider printing them directly if this behavior is desired.
		// fmt.Fprintln(s.writer, message)
	}
}

// IsActive returns true if the log scroller is currently running.
func (s *LogScroller) IsActive() bool {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.active
}

// --- Internal Methods ---
func (s *LogScroller) animate() {
	defer s.wg.Done()
	ticker := time.NewTicker(s.animationSpeed)
	defer ticker.Stop()

	for {
		// Check active state without holding lock during select.
		s.mu.Lock()
		isActive := s.active
		s.mu.Unlock()

		if !isActive { // LogScroller might have been stopped.
			s.drainTriggerSignalOnExit()
			return
		}

		select {
		case <-s.stopChan:
			s.drainTriggerSignalOnExit()
			// Drain logChan before final draw
			for done := false; !done; {
				select {
				case logMsg := <-s.logChan:
					s.mu.Lock()
					s.appendLogLine(logMsg)
					s.mu.Unlock()
				default: // No more logs in logChan
					done = true
				}
			}
			s.mu.Lock()
			s.draw(true) // Final draw
			s.mu.Unlock()
			return

		case logMsg := <-s.logChan:
			s.mu.Lock()
			s.appendLogLine(logMsg)
			// Optionally redraw immediately or wait for ticker/trigger.
			// For now, wait to batch updates with ticker/trigger.
			s.mu.Unlock()

		case <-ticker.C:
			s.mu.Lock()
			if s.active { // Re-check active under lock
				s.spinnerCursor = (s.spinnerCursor + 1) % len(s.spinnerChars)
				s.draw(false)
			}
			s.mu.Unlock()

		// NEW: Case for immediate redraw signal
		case <-s.triggerRedrawSignal:
			s.mu.Lock()
			if s.active { // Re-check active under lock
				// LogScroller character does not advance on triggered redraws, only on ticks
				s.draw(false)
			}
			s.mu.Unlock()
		}
	}
}

// Helper to drain triggerRedrawSignal before animate exits
func (s *LogScroller) drainTriggerSignalOnExit() {
	// This assumes triggerRedrawSignal is not closed, which it isn't in current design.
	// If it were closed, this would loop infinitely on a closed channel.
	// Since it's buffered, a few non-blocking reads are fine.
	for i := 0; i < cap(s.triggerRedrawSignal)+1; i++ { // cap+1 to be safe
		select {
		case <-s.triggerRedrawSignal:
		default:
			return
		}
	}
}

func (s *LogScroller) appendLogLine(message string) {
	// Assumes s.mu is locked by caller
	s.logBuffer = append(s.logBuffer, message)
	if s.linesToDisplay > 0 && len(s.logBuffer) > s.linesToDisplay {
		s.logBuffer = s.logBuffer[len(s.logBuffer)-s.linesToDisplay:]
	} else if s.linesToDisplay <= 0 && len(s.logBuffer) > 0 {
		s.logBuffer = []string{}
	}
}

func (s *LogScroller) clearLines() {
	// Assumes s.mu is locked by caller
	if s.linesPrinted > 0 {
		if s.enableColor {
			for i := 0; i < s.linesPrinted; i++ {
				fmt.Fprint(s.writer, cursorUp)
				fmt.Fprint(s.writer, clearLine)
			}
		}
	}
	s.linesPrinted = 0
}

func (s *LogScroller) draw(isFinal bool) {
	// Assumes s.mu is locked by caller
	s.clearLines()
	// ... (rest of draw method remains the same as previous version with padding) ...
	var sb strings.Builder
	currentLinesPrinted := 0

	// 1. LogScroller Character and Title Line
	charToPrint := ""
	colorForChar := ""

	if isFinal {
		switch s.finalStatus {
		case "success":
			charToPrint, colorForChar = s.charSuccess, s.colorSuccess
		case "error":
			charToPrint, colorForChar = s.charError, s.colorError
		default:
			charToPrint, colorForChar = s.charFinish, s.colorFinish
		}
	} else {
		charToPrint, colorForChar = s.spinnerChars[s.spinnerCursor], s.colorSpinner
	}

	if s.enableColor {
		sb.WriteString(colorForChar)
	}
	sb.WriteString(charToPrint)
	if s.title != "" {
		sb.WriteString(" ")
		if s.enableColor {
			sb.WriteString(s.colorTitle)
		}
		sb.WriteString(s.title)
	}
	if s.enableColor {
		sb.WriteString(colorResetCode)
	}
	sb.WriteString("\n")
	currentLinesPrinted++

	// 2. Log Scroller's Own Log Lines
	if s.linesToDisplay > 0 {
		var logsToPrint []string
		if len(s.logBuffer) > s.linesToDisplay {
			logsToPrint = s.logBuffer[len(s.logBuffer)-s.linesToDisplay:]
		} else {
			logsToPrint = s.logBuffer
		}

		numActualSpinnerLogs := len(logsToPrint)

		for _, logLine := range logsToPrint {
			cleanLine := strings.ReplaceAll(strings.TrimRight(logLine, "\n\r"), "\r", "")
			if s.enableColor {
				sb.WriteString(s.colorLogs)
			}
			sb.WriteString(cleanLine)
			if s.enableColor {
				sb.WriteString(colorResetCode)
			}
			sb.WriteString("\n")
			currentLinesPrinted++
		}
		if !isFinal {
			for i := numActualSpinnerLogs; i < s.linesToDisplay; i++ {
				sb.WriteString("\n")
				currentLinesPrinted++
			}
		}
	} else if isFinal && len(s.logBuffer) > 0 {
		for _, logLine := range s.logBuffer {
			cleanLine := strings.ReplaceAll(strings.TrimRight(logLine, "\n\r"), "\r", "")
			if s.enableColor {
				sb.WriteString(s.colorLogs)
			}
			sb.WriteString(cleanLine)
			if s.enableColor {
				sb.WriteString(colorResetCode)
			}
			sb.WriteString("\n")
			currentLinesPrinted++
		}
	}
	fmt.Fprint(s.writer, sb.String())
	s.linesPrinted = currentLinesPrinted
}

// --- Logrus Formatter ---
// LogrusLogScrollerFormatter remains the same. It calls s.clearLines(),
// and the NotifyingWriter will handle triggering the log scroller redraw after logrus writes.
type LogrusLogScrollerFormatter struct {
	LogScroller     *LogScroller
	TimestampFormat string
	Underlying      logrus.Formatter
	DisableColors   bool
}

func (f *LogrusLogScrollerFormatter) Format(entry *logrus.Entry) ([]byte, error) {
	if f.LogScroller != nil {
		f.LogScroller.mu.Lock() // Lock log scroller
		isActive := f.LogScroller.active
		if isActive {
			f.LogScroller.clearLines() // Erase current log scroller output
		}
		f.LogScroller.mu.Unlock() // Unlock log scroller
	}

	var b []byte
	var err error
	// ... (formatter logic as before) ...
	if f.Underlying != nil {
		currentBuffer := entry.Buffer
		entry.Buffer = bytes.NewBuffer(make([]byte, 0, 100))
		b, err = f.Underlying.Format(entry)
		entry.Buffer = currentBuffer
	} else {
		var logEntry bytes.Buffer
		if f.TimestampFormat != "" {
			logEntry.WriteString(entry.Time.Format(f.TimestampFormat))
			logEntry.WriteString(" ")
		}
		levelText := strings.ToUpper(entry.Level.String())
		colored := false
		if !f.DisableColors {
			switch entry.Level {
			case logrus.ErrorLevel, logrus.FatalLevel, logrus.PanicLevel:
				logEntry.WriteString("\033[31m")
				colored = true
			case logrus.WarnLevel:
				logEntry.WriteString("\033[33m")
				colored = true
			case logrus.InfoLevel:
				logEntry.WriteString("\033[36m")
				colored = true
			case logrus.DebugLevel, logrus.TraceLevel:
				logEntry.WriteString("\033[37m")
				colored = true
			}
		}
		logEntry.WriteString("[" + levelText + "] ")
		if colored {
			logEntry.WriteString(colorResetCode)
		}
		logEntry.WriteString(entry.Message)
		logEntry.WriteString("\n")
		b = logEntry.Bytes()
	}
	return b, err
}
