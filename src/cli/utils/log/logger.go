package log

import (
	"os"
	"time"

	"github.com/sirupsen/logrus"
)

// Note, any changes here should also be applied to SetupLogging and TeardownLogging
var log = &logrus.Logger{
	Out: NewNotifyingWriter(os.Stderr, nil),
	Formatter: &LogrusLogScrollerFormatter{
		LogScroller:     nil,
		TimestampFormat: time.RFC3339,
	},
	Hooks: make(logrus.LevelHooks),
	Level: logrus.DebugLevel,
}

type logFunc func(args ...interface{})
type logfFunc func(format string, args ...interface{})

var Fatal logFunc = log.Fatal
var Fatalf logfFunc = log.Fatalf

var Error logFunc = log.Error
var Errorf logfFunc = log.Errorf

var Warn logFunc = log.Warn
var Warnf logfFunc = log.Warnf

var Info logFunc = log.Info
var Infof logfFunc = log.Infof

var Debug logFunc = log.Debug
var Debugf logfFunc = log.Debugf

var Trace logFunc = log.Trace
var Tracef logfFunc = log.Tracef
