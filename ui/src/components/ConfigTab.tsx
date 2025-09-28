import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

function ConfigTab() {
  const [invertX, setInvertX] = useState(false);
  const [invertY, setInvertY] = useState(false);

  async function saveConfig() {
    await invoke("update_config", {
      config: { invertX, invertY },
    });
  }

  return (
    <div>
      <h2>Configuração</h2>
      <div className="form-group">
        <label>
          <input
            type="checkbox"
            checked={invertX}
            onChange={(e) => setInvertX(e.target.checked)}
          />
          Inverter eixo X
        </label>
      </div>
      <div className="form-group">
        <label>
          <input
            type="checkbox"
            checked={invertY}
            onChange={(e) => setInvertY(e.target.checked)}
          />
          Inverter eixo Y
        </label>
      </div>

      <button onClick={saveConfig}>Salvar</button>
    </div>
  );
}

export default ConfigTab;
