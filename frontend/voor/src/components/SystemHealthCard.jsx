import { useEffect, useMemo, useState } from "react";
import { fetchSystemHealth } from "../api.js";

const healthIconByStatus = {
  healthy: "check_circle",
  warning: "warning",
  degraded: "error",
  down: "error",
};

function formatDuration(milliseconds) {
  const value = Number(milliseconds);
  if (!Number.isFinite(value) || value < 0) {
    return "";
  }

  const totalSeconds = Math.floor(value / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }

  if (minutes > 0) {
    return `${minutes}m ${seconds}s`;
  }

  return `${seconds}s`;
}

function normalizeHealth(health) {
  return String(health || "unknown").toLowerCase();
}

export function SystemHealthCard({ copy }) {
  const [state, setState] = useState({
    status: "loading",
    data: null,
    error: null,
  });

  useEffect(() => {
    let active = true;

    setState({ status: "loading", data: null, error: null });
    fetchSystemHealth()
      .then((data) => {
        if (active) {
          setState({ status: "ready", data, error: null });
        }
      })
      .catch((error) => {
        if (active) {
          setState({ status: "unavailable", data: null, error: error.message });
        }
      });

    return () => {
      active = false;
    };
  }, []);

  const services = useMemo(() => {
    if (!Array.isArray(state.data?.services)) {
      return [];
    }

    return state.data.services;
  }, [state.data]);

  const overallStatus = normalizeHealth(state.data?.overall_status);
  const statusLabel = state.status === "ready" ? copy.statuses[overallStatus] ?? overallStatus : copy.loading;
  const latestLog = Array.isArray(state.data?.recent_logs) ? state.data.recent_logs[0] : null;

  return (
    <section className="system-health-card" aria-label={copy.title}>
      <header className="system-health-header">
        <div>
          <p className="label-caps">{copy.eyebrow}</p>
          <h2>{copy.title}</h2>
        </div>
        <span className={`health-badge health-${state.status === "ready" ? overallStatus : state.status}`}>
          <span className="material-symbols-outlined" aria-hidden="true">
            {state.status === "ready" ? healthIconByStatus[overallStatus] ?? "help" : "sync"}
          </span>
          {statusLabel}
        </span>
      </header>

      {state.status === "ready" ? (
        <>
          <div className="system-health-summary">
            <div>
              <span>{copy.uptime}</span>
              <strong>{formatDuration(state.data?.uptime_ms) || copy.noData}</strong>
            </div>
            <div>
              <span>{copy.services}</span>
              <strong>{services.length}</strong>
            </div>
          </div>

          <div className="service-health-list">
            {services.map((service) => {
              const serviceHealth = normalizeHealth(service.health);
              return (
                <article className="service-health-row" key={service.service}>
                  <span className={`service-health-dot health-${serviceHealth}`} aria-hidden="true" />
                  <div>
                    <strong>{service.service}</strong>
                    <span>{service.status}</span>
                  </div>
                  <p>{service.last_message}</p>
                </article>
              );
            })}
          </div>

          {latestLog ? (
            <footer className="system-health-log">
              <span>{copy.latestEvent}</span>
              <strong>{latestLog.event}</strong>
              <p>{latestLog.message}</p>
            </footer>
          ) : null}
        </>
      ) : (
        <p className="system-health-message">
          {state.status === "loading" ? copy.loading : state.error || copy.noData}
        </p>
      )}
    </section>
  );
}
