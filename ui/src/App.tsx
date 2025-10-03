import { useState } from "react";
import StatusTab from "./components/StatusTab";
import ConfigTab from "./components/ConfigTab";
import "./App.css";

function App() {
  const [activeTab, setActiveTab] = useState<"status" | "config">("status");

  return (
    <main className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 p-6">
      <div className="max-w-4xl mx-auto bg-white rounded-2xl shadow-lg overflow-hidden">
        {/* Header */}
        <div className="bg-gradient-to-r from-blue-600 to-indigo-700 p-6">
          <h1 className="text-2xl font-bold text-white text-center">
            TableZ Controller
          </h1>
        </div>

        {/* Tabs Navigation */}
        <div className="border-b border-gray-200">
          <nav className="flex space-x-1 px-6 pt-4" aria-label="Tabs">
            <button
              onClick={() => setActiveTab("status")}
              className={`px-4 py-2.5 text-sm font-medium rounded-t-lg transition-all duration-200 ${
                activeTab === "status"
                  ? "bg-white text-blue-600 border-t border-l border-r border-gray-200"
                  : "text-gray-500 hover:text-gray-700 hover:bg-gray-50"
              }`}
            >
              📊 Status
            </button>
            <button
              onClick={() => setActiveTab("config")}
              className={`px-4 py-2.5 text-sm font-medium rounded-t-lg transition-all duration-200 ${
                activeTab === "config"
                  ? "bg-white text-blue-600 border-t border-l border-r border-gray-200"
                  : "text-gray-500 hover:text-gray-700 hover:bg-gray-50"
              }`}
            >
              ⚙️ Configuração
            </button>
          </nav>
        </div>

        {/* Tab Content */}
        <section className="p-6 bg-white">
          {activeTab === "status" && <StatusTab />}
          {activeTab === "config" && <ConfigTab />}
        </section>
      </div>
    </main>
  );
}

export default App;