import { useEffect, useState } from "react";
import { listMacros, deleteMacro, enableMacro, disableMacro, type MacroSummary } from "@/services/MacroService";
import { formattedKey } from "@/services/KeyListService";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Trash2, RefreshCw, Power, Pencil } from "lucide-react";
import { useNavigate } from "react-router-dom";

export function HomePage() {
  const [macros, setMacros] = useState<MacroSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  async function loadMacros() {
    setLoading(true);
    try {
      const result = await listMacros();
      setMacros(result);
    } catch (e) {
      console.error("Error loading macros:", e);
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    loadMacros();
  }, []);

  async function handleDelete(triggerKey: string) {
    try {
      const success = await deleteMacro(triggerKey);
      if (success) {
        setMacros((prev) => prev.filter((m) => m.triggerKey !== triggerKey));
      }
    } catch (e) {
      console.error("Error deleting macro:", e);
    }
  }

  async function handleToggleEnabled(triggerKey: string, currentEnabled: boolean) {
    try {
      const success = currentEnabled 
        ? await disableMacro(triggerKey)
        : await enableMacro(triggerKey);
      
      if (success) {
        setMacros((prev) => 
          prev.map((m) => 
            m.triggerKey === triggerKey ? { ...m, enabled: !currentEnabled } : m
          )
        );
      }
    } catch (e) {
      console.error("Error toggling macro:", e);
    }
  }

  function handleEdit(triggerKey: string) {
    navigate("/create-macro", { state: { editTriggerKey: triggerKey } });
  }

  return (
    <div className="w-full max-w-4xl mx-auto p-6 flex flex-col gap-6">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="font-semibold text-2xl">Saved Macros</h1>
          <p className="text-sm text-muted-foreground">
            Manage your configured macros
          </p>
        </div>
        <Button variant="outline" size="sm" onClick={loadMacros} disabled={loading}>
          <RefreshCw className={`size-4 mr-2 ${loading ? "animate-spin" : ""}`} />
          Refresh
        </Button>
      </header>

      {loading && macros.length === 0 ? (
        <div className="bg-primary-foreground rounded-md p-8 text-center text-muted-foreground">
          Loading macros...
        </div>
      ) : macros.length === 0 ? (
        <div className="bg-primary-foreground rounded-md p-8 text-center text-muted-foreground">
          No macros saved yet. Go to "Create Macro" to add one.
        </div>
      ) : (
        <div className="flex flex-col gap-3">
          {macros.map((macro) => (
            <Card key={macro.triggerKey} className="p-4 flex items-center justify-between">
              <div className="flex items-center gap-4">
                <div className="flex flex-col">
                  <span className="font-semibold text-lg">
                    {formattedKey(macro.triggerKey)}
                  </span>
                  <span className="text-sm text-muted-foreground">
                    {macro.mode} · {macro.actionsCount} action{macro.actionsCount !== 1 ? "s" : ""}
                  </span>
                </div>
                <div className="flex gap-2">
                  {macro.blockOriginalEvent && (
                    <span className="text-xs px-2 py-0.5 rounded bg-muted text-muted-foreground">
                      Blocks original
                    </span>
                  )}
                  {macro.ignoreInjectedKeys && (
                    <span className="text-xs px-2 py-0.5 rounded bg-muted text-muted-foreground">
                      Ignores injected
                    </span>
                  )}
                  <span className={`text-xs px-2 py-0.5 rounded ${macro.enabled ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'}`}>
                    {macro.enabled ? 'Enabled' : 'Disabled'}
                  </span>
                </div>
              </div>
              <div className="flex gap-2">
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => handleEdit(macro.triggerKey)}
                  title="Edit macro"
                >
                  <Pencil className="size-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => handleToggleEnabled(macro.triggerKey, macro.enabled)}
                  className={macro.enabled ? "text-green-600 hover:text-green-700" : "text-red-600 hover:text-red-700"}
                >
                  <Power className="size-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => handleDelete(macro.triggerKey)}
                  className="text-destructive hover:text-destructive"
                >
                  <Trash2 className="size-4" />
                </Button>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
