import { useState, useEffect } from "react";
import MacroForm, {
  macroFormSchema,
  type MacroFormValues,
} from "./components/MacroForm";
import ActionsList from "./components/ActionsList";
import AddActionForm from "./components/AddActionForm";
import { Button } from "@/components/ui/button";
import { Action } from "@/model/Action";
import { Mode } from "@/model/Mode";
import { Macro } from "@/model/Macro";
import { Save } from "lucide-react";
import { addMacro, getMacro, updateMacro } from "@/services/MacroService";
import { useNavigate, useLocation } from "react-router-dom";

type ActionItem = { id: string; action: Action };

const initialConfig: MacroFormValues = {
  triggerKey: "",
  mode: "",
  blockOriginalEvent: true,
  ignoreInjectedKeys: true,
  enabled: true,
};

function newId() {
  return crypto.randomUUID();
}

export function CreateMacroPage() {
  const [config, setConfig] = useState<MacroFormValues>(initialConfig);
  const [actions, setActions] = useState<ActionItem[]>([]);
  const [saving, setSaving] = useState(false);
  const [loading, setLoading] = useState(false);
  const [originalTriggerKey, setOriginalTriggerKey] = useState<string | null>(null);
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    const editTriggerKey = location.state?.editTriggerKey;
    if (editTriggerKey) {
      setLoading(true);
      setOriginalTriggerKey(editTriggerKey);
      getMacro(editTriggerKey).then(macro => {
        if (macro) {
          setConfig({
            triggerKey: macro.triggerKey,
            mode: macro.mode,
            blockOriginalEvent: macro.blockOriginalKey,
            ignoreInjectedKeys: macro.ignoreInjectedKeys,
            enabled: macro.enabled,
          });
          setActions(macro.actions.map(action => ({ id: newId(), action })));
        }
        setLoading(false);
      });
    }
  }, [location.state]);

  function handleAdd(actions: Action[]) {
    const newItems = actions.map(action => ({ id: newId(), action }));
    setActions((prev) => [...prev, ...newItems]);
  }

  function handleRemove(id: string) {
    setActions((prev) => prev.filter((a) => a.id !== id));
  }

  function handleReorder(newActions: ActionItem[]) {
    setActions(newActions);
  }

  function navigateToHome() {
    navigate("/");
  }

  async function handleSave() {
    const result = macroFormSchema.safeParse(config);
    if (!result.success) {
      console.error("Invalid config:", result.error);
      return;
    }
    const macro: Macro = {
      triggerKey: result.data.triggerKey,
      mode: result.data.mode as Mode,
      actions: actions.map((a) => a.action),
      blockOriginalKey: result.data.blockOriginalEvent,
      ignoreInjectedKeys: result.data.ignoreInjectedKeys,
      enabled: result.data.enabled,
    };
    setSaving(true);
    try {
      const success = originalTriggerKey
        ? await updateMacro(originalTriggerKey, macro)
        : await addMacro(macro);
      if (success) {
        setConfig(initialConfig);
        setActions([]);
        setOriginalTriggerKey(null);
      }
      setSaving(false);
      navigateToHome();
    } catch (e) {
      console.error("Error saving macro:", e);
      setSaving(false);
    }
  }

  const isValid = !!config.triggerKey && !!config.mode && actions.length > 0;
  const isEditing = !!originalTriggerKey;

  return (
    <div className="w-full max-w-6xl mx-auto p-6 flex flex-col gap-8">
      {loading ? (
        <div className="flex items-center justify-center py-12">
          <p className="text-muted-foreground">Loading macro...</p>
        </div>
      ) : (
        <>
          <section>
            <header className="mb-3">
              <h2 className="text-xl font-semibold">
                {isEditing ? "Edit Macro" : "Configuration"}
              </h2>
              <p className="text-sm text-muted-foreground">
                {isEditing 
                  ? `Editing macro for ${config.triggerKey || "key"}` 
                  : "Set up how the macro triggers and behaves"}
              </p>
            </header>
            <MacroForm values={config} onChange={setConfig} />
          </section>

          <section className="grid grid-cols-1 lg:grid-cols-3 gap-6 min-h-[400px]">
            <div className="lg:col-span-2 flex flex-col">
              <header className="mb-3">
                <h2 className="text-xl font-semibold">Actions</h2>
                <p className="text-sm text-muted-foreground">
                  Drag to reorder the sequence of actions
                </p>
              </header>
              <div className="flex-1">
                <ActionsList
                  actions={actions}
                  onReorder={handleReorder}
                  onRemove={handleRemove}
                />
              </div>
            </div>

            <div className="flex flex-col">
              <header className="mb-3">
                <h2 className="text-xl font-semibold">Add</h2>
                <p className="text-sm text-muted-foreground">Insert a new action</p>
              </header>
              <div className="flex-1">
                <AddActionForm onAdd={handleAdd} />
              </div>
            </div>
          </section>

          <div className="flex justify-end">
            <Button onClick={handleSave} disabled={!isValid || saving}>
              <Save className="size-4" />
              {saving ? "Saving..." : isEditing ? "Update Macro" : "Save Macro"}
            </Button>
          </div>
        </>
      )}
    </div>
  );
}
