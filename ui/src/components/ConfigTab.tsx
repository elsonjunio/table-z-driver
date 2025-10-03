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

  const [isSaving, setIsSaving] = useState(false);
  const [saveStatus, setSaveStatus] = useState<"idle" | "success" | "error">("idle");

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
        setConfig(cfg);

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
    setIsSaving(true);
    setSaveStatus("idle");
    
    try {
      const mappedButtons = buttons.map(([k1, k2]) =>
        [k1, k2].filter(k => k !== "None").join("+")
      );
      
      const newConfig: Config = {
        ...config,
        actions: {
          ...config.actions,
          tablet_buttons: mappedButtons,
        },
      };

      await invoke("update_config", { config: newConfig });
      setSaveStatus("success");
      
      // Reset success status after 3 seconds
      setTimeout(() => setSaveStatus("idle"), 3000);
    } catch (e) {
      console.error("Erro ao salvar:", e);
      setSaveStatus("error");
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <div className="space-y-6">
      <h2 className="text-xl font-semibold text-gray-800">Configurações</h2>
      
      {/* Configurações Gerais */}
      <div className="bg-gray-50 p-4 rounded-lg border border-gray-200">
        <h3 className="text-lg font-medium text-gray-700 mb-4">Configurações de Eixo</h3>
        
        <div className="space-y-3">
          <label className="flex items-center space-x-3 p-3 bg-white rounded-lg border border-gray-300 hover:border-blue-400 transition-colors cursor-pointer">
            <input
              type="checkbox"
              checked={config.settings.swap_direction_x}
              onChange={(e) =>
                setConfig((prev) => ({
                  ...prev,
                  settings: { ...prev.settings, swap_direction_x: e.target.checked },
                }))
              }
              className="w-4 h-4 text-blue-600 rounded focus:ring-blue-500"
            />
            <span className="text-gray-700 font-medium">Inverter Direção do Eixo X</span>
          </label>
          
          <label className="flex items-center space-x-3 p-3 bg-white rounded-lg border border-gray-300 hover:border-blue-400 transition-colors cursor-pointer">
            <input
              type="checkbox"
              checked={config.settings.swap_direction_y}
              onChange={(e) =>
                setConfig((prev) => ({
                  ...prev,
                  settings: { ...prev.settings, swap_direction_y: e.target.checked },
                }))
              }
              className="w-4 h-4 text-blue-600 rounded focus:ring-blue-500"
            />
            <span className="text-gray-700 font-medium">Inverter Direção do Eixo Y</span>
          </label>
        </div>
      </div>

      {/* Mapeamento de Botões */}
      <div className="bg-gray-50 p-4 rounded-lg border border-gray-200">
        <h3 className="text-lg font-medium text-gray-700 mb-4">Mapeamento de Botões</h3>
        
        <div className="space-y-3">
          {buttons.map((combo, idx) => (
            <div
              key={idx}
              className="flex items-center space-x-4 p-3 bg-white rounded-lg border border-gray-300 hover:border-blue-400 transition-colors"
            >
              <span className="w-20 text-sm font-medium text-gray-600">
                Botão {idx + 1}
              </span>
              
              <select
                value={combo[0]}
                onChange={(e) => updateButton(idx, 0, e.target.value)}
                className="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                {KEY_OPTIONS.map((k) => (
                  <option key={k} value={k}>
                    {k.replace('KEY_', '')}
                  </option>
                ))}
              </select>
              
              <span className="text-gray-400">+</span>
              
              <select
                value={combo[1]}
                onChange={(e) => updateButton(idx, 1, e.target.value)}
                className="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              >
                {KEY_OPTIONS.map((k) => (
                  <option key={k} value={k}>
                    {k.replace('KEY_', '')}
                  </option>
                ))}
              </select>
            </div>
          ))}
        </div>
      </div>

      {/* Save Button */}
      <div className="flex items-center space-x-4">
        <button 
          onClick={saveConfig}
          disabled={isSaving}
          className={`px-6 py-3 rounded-lg font-medium transition-all duration-200 ${
            isSaving 
              ? "bg-gray-400 cursor-not-allowed" 
              : "bg-blue-600 hover:bg-blue-700 text-white shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
          }`}
        >
          {isSaving ? "Salvando..." : "💾 Salvar Configurações"}
        </button>
        
        {saveStatus === "success" && (
          <div className="flex items-center space-x-2 text-green-600">
            <span>✅ Configurações salvas com sucesso!</span>
          </div>
        )}
        
        {saveStatus === "error" && (
          <div className="flex items-center space-x-2 text-red-600">
            <span>❌ Erro ao salvar configurações</span>
          </div>
        )}
      </div>
    </div>
  );
}

export default ConfigTab;
