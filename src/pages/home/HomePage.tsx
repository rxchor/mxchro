import { useEffect, useState } from "react";
import { listMacros, deleteMacro, type MacroSummary } from "@/services/MacroService";
import { formattedKey } from "@/services/KeyListService";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Trash2, RefreshCw } from "lucide-react";

export function HomePage() {
  const [macros, setMacros] = useState<MacroSummary[]>([]);
  const [loading, setLoading] = useState(true);

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
                </div>
              </div>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => handleDelete(macro.triggerKey)}
                className="text-destructive hover:text-destructive"
              >
                <Trash2 className="size-4" />
              </Button>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
