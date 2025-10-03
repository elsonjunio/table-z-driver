import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

interface PenStatus {
  Pen: {
    x: number;
    y: number;
    pressure: number;
    touch: boolean;
  }
}

interface BtnStatus {
  Btn: {
    key: number;
    pressed: boolean;
    index: number;
  }
}

function StatusTab() {
  const [pen_status, setPenStatus] = useState<PenStatus>({
    Pen: {
      x: 0,
      y: 0,
      pressure: 0,
      touch: false,
    }
  });

  const [btn_status, setBtnStatus] = useState<BtnStatus>({
    Btn: {
      key: 0,
      pressed: false,
      index: 0
    }
  });

  useEffect(() => {
    const unlisten = listen<PenStatus>("pen_event", (event) => {
      setPenStatus(event.payload);  
    });
    return () => {
      unlisten.then(f => f());
    };
  }, []);

  useEffect(() => {
    const unlisten = listen<BtnStatus>("btn_event", (event) => {
      setBtnStatus(event.payload);  
    });
    return () => {
      unlisten.then(f => f());
    };
  }, []);

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold text-gray-800">Status da Mesa Digitalizadora</h2>
      
      {/* Status Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-blue-50 p-4 rounded-lg border border-blue-200">
          <div className="text-sm text-blue-600 font-medium">Posição</div>
          <div className="text-2xl font-bold text-blue-800 mt-1">
            {pen_status.Pen.x}, {pen_status.Pen.y}
          </div>
        </div>
        
        <div className="bg-green-50 p-4 rounded-lg border border-green-200">
          <div className="text-sm text-green-600 font-medium">Pressão</div>
          <div className="text-2xl font-bold text-green-800 mt-1">
            {pen_status.Pen.pressure}
          </div>
        </div>
        
        <div className={`p-4 rounded-lg border ${
          btn_status.Btn.pressed 
            ? "bg-red-50 border-red-200" 
            : "bg-gray-50 border-gray-200"
        }`}>
          <div className={`text-sm font-medium ${
            btn_status.Btn.pressed ? "text-red-600" : "text-gray-600"
          }`}>
            Botão {btn_status.Btn.pressed ? "Pressionado" : "Livre"}
          </div>
          <div className={`text-2xl font-bold mt-1 ${
            btn_status.Btn.pressed ? "text-red-800" : "text-gray-800"
          }`}>
            {btn_status.Btn.pressed ? `Botão ${btn_status.Btn.index + 1}` : "Nenhum"}
          </div>
        </div>
      </div>

      {/* Touch Indicator */}
      <div className="flex items-center space-x-3 p-4 bg-yellow-50 rounded-lg border border-yellow-200">
        <div className={`w-3 h-3 rounded-full ${
          pen_status.Pen.touch ? "bg-green-500 animate-pulse" : "bg-gray-400"
        }`}></div>
        <span className="text-sm font-medium text-yellow-800">
          {pen_status.Pen.touch ? "Toque ativo" : "Toque inativo"}
        </span>
      </div>

      {/* Visualization Canvas */}
      <div className="bg-slate-50 p-4 rounded-lg border border-gray-200">
        <h3 className="text-lg font-medium text-gray-700 mb-4">Visualização</h3>
        <div className="relative bg-white rounded-lg border border-gray-300 p-4">
          <svg width="100%" height="200" className="rounded">
            {/* Grade de fundo */}
            <defs>
              <pattern id="grid" width="50" height="50" patternUnits="userSpaceOnUse">
                <path d="M 50 0 L 0 0 0 50" fill="none" stroke="#e5e7eb" strokeWidth="1"/>
              </pattern>
            </defs>
            <rect width="100%" height="100%" fill="url(#grid)" />
            
            {/* Ponto da caneta */}
            <circle
              cx={(pen_status.Pen.x * 0.073242188)}
              cy={(pen_status.Pen.y * 0.048828125)}
              r={Math.max(5, pen_status.Pen.pressure / 100)}
              fill={pen_status.Pen.touch ? "#3b82f6" : "#9ca3af"}
              className="transition-all duration-150 ease-out"
              opacity={0.8}
            />
            
            {/* Anel de pressão */}
            {pen_status.Pen.touch && (
              <circle
                cx={(pen_status.Pen.x * 0.073242188)}
                cy={(pen_status.Pen.y * 0.048828125)}
                r={Math.max(8, pen_status.Pen.pressure / 80)}
                fill="none"
                stroke="#3b82f6"
                strokeWidth="2"
                opacity={0.4}
              />
            )}
          </svg>
          
          {/* Coordenadas no canto */}
          <div className="absolute bottom-2 right-2 bg-black bg-opacity-70 text-white text-xs px-2 py-1 rounded">
            X: {pen_status.Pen.x} | Y: {pen_status.Pen.y}
          </div>
        </div>
      </div>
    </div>
  );
}

export default StatusTab;