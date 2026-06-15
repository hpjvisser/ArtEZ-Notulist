import { useEffect, useRef } from "react";
import type { LogEvent } from "../types";

export function LogPanel({ logs }: { logs: LogEvent[] }) {
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  return (
    <div className="logpanel">
      <div className="logpanel__title">Logvenster</div>
      <div className="logpanel__body">
        {logs.length === 0 && (
          <div className="logpanel__empty">Nog geen activiteit.</div>
        )}
        {logs.map((l, i) => (
          <div key={i} className={`logline logline--${l.level}`}>
            <span className="logline__time">
              {new Date(l.timestamp).toLocaleTimeString("nl-NL")}
            </span>
            <span className="logline__msg">{l.message}</span>
          </div>
        ))}
        <div ref={endRef} />
      </div>
    </div>
  );
}
