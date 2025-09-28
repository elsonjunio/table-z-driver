import { useState } from "react";
import StatusTab from "./components/StatusTab";
import ConfigTab from "./components/ConfigTab";
import "./App.css";

function App() {
  const [activeTab, setActiveTab] = useState<"status" | "config">("status");

  return (
    <main className="container">
      <header className="tabs">
        <button
          className={activeTab === "status" ? "active" : ""}
          onClick={() => setActiveTab("status")}
        >
          Status
        </button>
        <button
          className={activeTab === "config" ? "active" : ""}
          onClick={() => setActiveTab("config")}
        >
          Configuração
        </button>
      </header>

      <section className="tab-content">
        {activeTab === "status" && <StatusTab />}
        {activeTab === "config" && <ConfigTab />}
      </section>
    </main>
  );
}

export default App;
