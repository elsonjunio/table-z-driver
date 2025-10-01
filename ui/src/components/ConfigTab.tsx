import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const KEY_OPTIONS = [
  "None",
  "KEY_LEFTCTRL",
  "KEY_LEFTSHIFT",
  "KEY_LEFTALT",
  "KEY_A", "KEY_B", "KEY_C", "KEY_D", "KEY_E", "KEY_F", "KEY_G",
  "KEY_H", "KEY_I", "KEY_J", "KEY_K", "KEY_L", "KEY_M", "KEY_N", "KEY_O", "KEY_P",
  "KEY_Q", "KEY_R", "KEY_S", "KEY_T", "KEY_U", "KEY_V", "KEY_W", "KEY_X", "KEY_Y", "KEY_Z",
  "KEY_ESC", "KEY_TAB", "KEY_ENTER", "KEY_SPACE",
  "KEY_UP", "KEY_DOWN", "KEY_LEFT", "KEY_RIGHT",
  "KEY_F1","KEY_F2","KEY_F3","KEY_F4","KEY_F5","KEY_F6","KEY_F7","KEY_F8","KEY_F9","KEY_F10","KEY_F11","KEY_F12",
];

type Config = {
    xinput_name: String;
    vendor_id: number;
    product_id: number;
    interface: number;
    pen: PenConfig;
    actions: ActionsConfig;
    settings: SettingsConfig;
}

type PenConfig = {
    max_x: number;
    max_y: number;
    max_pressure: number;
    resolution_x: number;
    resolution_y: number;
}

type ActionsConfig = {
    pen: String;
    stylus: String;
    pen_touch: String;
    tablet_buttons: Array<string>;
}

type SettingsConfig = {
    swap_axis: boolean;
    swap_direction_x: boolean;
    swap_direction_y: boolean;
}

function ConfigTab() {
  const [config, setConfig] = useState<Config>({
    xinput_name: "",
    vendor_id: 0,
    product_id: 0,
    interface: 0,
    pen: { max_x: 0, max_y: 0, max_pressure: 0, resolution_x: 0, resolution_y: 0 },
    actions: { pen: "", stylus: "", pen_touch: "", tablet_buttons: [] },
    settings: { swap_axis: false, swap_direction_x: false, swap_direction_y: false },
  });


  const [buttons, setButtons] = useState<Array<[string, string]>>(
    Array(8).fill(["None", "None"])
  );

  function updateButton(index: number, pos: 0 | 1, value: string) {
    setButtons((prev) => {
      const newButtons = [...prev];
      newButtons[index] = [...newButtons[index]] as [string, string];
      newButtons[index][pos] = value;
      return newButtons;
    });
  }


  useEffect(() => {
    async function loadConfig() {
      try {
        const cfg = await invoke<Config>("get_config");
        console.log(cfg);

        setConfig(cfg);

        // converte ["Ctrl+Z", "Alt+Tab", "A", ""] -> [["Ctrl","Z"],["Alt","Tab"],["A","None"],["None","None"],...]
        const parsed = (cfg.actions.tablet_buttons || []).map((combo: string) => {
          if (!combo) return ["None", "None"];
          const parts = combo.split("+");
          return [parts[0] || "None", parts[1] || "None"];
        });

        while (parsed.length < 8) parsed.push(["None", "None"]);
        setButtons(parsed as Array<[string, string]>);
      } catch (e) {
        console.error("Erro ao carregar config:", e);
      }
    }
    loadConfig();
  }, []);

async function saveConfig() {
  const mappedButtons = buttons.map(([k1, k2]) =>
    [k1, k2].filter(k => k !== "None").join("+")
  );
  
    const newConfig: Config = {
      ...config,
      actions: {
        ...config.actions,
        tablet_buttons: mappedButtons,
      },
      settings: {
        ...config.settings,
        swap_direction_x: config.settings.swap_direction_x,
        swap_direction_y: config.settings.swap_direction_y,
      },
    };

    await invoke("update_config", { config: newConfig });

  }

  return (
    <div>
      <h2>Configuração</h2>
      <div className="form-group">
        <label>
          <input
            type="checkbox"
            checked={config.settings.swap_direction_x}
            onChange={(e) =>
              setConfig((prev) => ({
                ...prev,
                settings: { ...prev.settings, swap_direction_x: e.target.checked },
              }))
            }
          />
          Inverter eixo X
        </label>
      </div>
      <div className="form-group">
        <label>
          <input
            type="checkbox"
            checked={config.settings.swap_direction_y}
            onChange={(e) =>
              setConfig((prev) => ({
                ...prev,
                settings: { ...prev.settings, swap_direction_y: e.target.checked },
              }))
            }
          />
          Inverter eixo Y
        </label>
      </div>

      <div>
        <h3>Mapeamento de Botões</h3>
        {buttons.map((combo, idx) => (
          <div
            key={idx}
            style={{ display: "flex", gap: "1rem", marginBottom: "0.5rem" }}
          >
            <span>Botão {idx + 1}</span>
            <select
              value={combo[0]}
              onChange={(e) => updateButton(idx, 0, e.target.value)}
            >
              {KEY_OPTIONS.map((k) => (
                <option key={k} value={k}>
                  {k}
                </option>
              ))}
            </select>
            <select
              value={combo[1]}
              onChange={(e) => updateButton(idx, 1, e.target.value)}
            >
              {KEY_OPTIONS.map((k) => (
                <option key={k} value={k}>
                  {k}
                </option>
              ))}
            </select>
          </div>
        ))}
      </div>

      <button onClick={saveConfig}>Salvar</button>
    </div>
  );
}

export default ConfigTab;
