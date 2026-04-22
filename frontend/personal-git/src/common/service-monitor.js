const DEFAULT_HISTORY_LIMIT = 100;

export function createServiceMonitor(serviceName, historyLimit = DEFAULT_HISTORY_LIMIT) {
  const history = [];
  let currentStatus = {
    service: serviceName,
    health: "starting",
    status: "booting",
    lastMessage: "Service monitor created",
    updatedAt: Date.now(),
  };

  const write = (level, event, message, details = undefined) => {
    const entry = {
      timestamp: Date.now(),
      service: serviceName,
      level,
      event,
      message,
      details,
    };

    console[level](`[${level.toUpperCase()}][${serviceName}][${event}] ${message}`, details ?? "");

    history.unshift(entry);
    if (history.length > historyLimit) {
      history.pop();
    }

    return entry;
  };

  return {
    setStatus(health, status, message) {
      currentStatus = {
        service: serviceName,
        health,
        status,
        lastMessage: message,
        updatedAt: Date.now(),
      };
      write("info", "status-update", message, currentStatus);
      return currentStatus;
    },
    logInfo(event, message, details) {
      return write("info", event, message, details);
    },
    logWarn(event, message, details) {
      return write("warn", event, message, details);
    },
    logError(event, message, details) {
      return write("error", event, message, details);
    },
    snapshot() {
      return {
        status: currentStatus,
        recentLogs: [...history],
      };
    },
  };
}
