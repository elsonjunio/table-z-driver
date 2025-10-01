import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

interface PenStatus {
  Pen:{
  x: number;
  y: number;
  pressure: number;
  touch: boolean;
  }
}

interface BtnStatus {
  Btn:{
  key: number,
  pressed: boolean,
  index: number,
  }
}

function StatusTab() {
  const [pen_status, setPenStatus] = useState<PenStatus>({
    Pen:{
    x: 0,
    y: 0,
    pressure: 0,
    touch: false,
    }
  });

  const [btn_status, setBtnStatus] = useState<BtnStatus>({
    Btn:{
    key: 0,
    pressed: false,
    index: 0
    }
  });

  useEffect(() => {
    const unlisten = listen<PenStatus>("pen_event", (event) => {
      setPenStatus(event.payload);  
      console.log(event.payload);
    });
    return () => {
      unlisten.then(f => f());
    };
  }, []);

  useEffect(() => {
    const unlisten = listen<BtnStatus>("btn_event", (event) => {
      setBtnStatus(event.payload);  
      console.log(event.payload);
    });
    return () => {
      unlisten.then(f => f());
    };
  }, []);

  return (
    <div>
      <h2>Status da Mesa</h2>
      <div className="status-grid">
        <div>
          <strong>Posição:</strong> {pen_status.Pen.x}, {pen_status.Pen.y}
        </div>
        <div>
          <strong>Pressão:</strong> {pen_status.Pen.pressure}
        </div>
        <div>
          <strong>Botões:</strong>{" "}
           {btn_status.Btn.pressed ? btn_status.Btn.index : "Nenhum"} 
        </div>
      </div>

      <svg width="300" height="200" style={{ border: "1px solid #ccc" }}>
        {/*  300/4096 = 0.073242188 */}
        {/* 200/4096 = 0.048828125 */}
        <circle
          cx={(pen_status.Pen.x * 0.073242188)}
          cy={(pen_status.Pen.y * 0.048828125)}
          r={5 + pen_status.Pen.pressure / 200}
          fill="blue"
        />
      </svg>
    </div>
  );
}

export default StatusTab;
