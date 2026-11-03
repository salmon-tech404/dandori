import { t } from "../../core/i18n";
import { Button } from "../../ui/components/Button";
import { Card } from "../../ui/components/Card";

export interface AgentRoleDef {
  id: string;
  name: string;
  provider: string;
  letter: string;
  color: string;
  bg: string;
}

export const KNOWLEDGE_ROLES: AgentRoleDef[] = [
  { id: "pm", name: "PM", provider: "Claude", letter: "C", color: "var(--or)", bg: "var(--orb)" },
  { id: "worker", name: "Worker", provider: "Codex", letter: "K", color: "var(--tl)", bg: "var(--tlb)" },
  { id: "test", name: "Agent test", provider: "Gemini", letter: "G", color: "var(--bl)", bg: "var(--blb)" },
  { id: "doc", name: "Agent tài liệu", provider: "Claude", letter: "C", color: "var(--or)", bg: "var(--orb)" },
];

const badgeStyle = (color: string, bg: string): React.CSSProperties => ({
  width: "22px",
  height: "22px",
  borderRadius: "50%",
  backgroundColor: bg,
  color,
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  fontSize: "11px",
  fontWeight: 700,
});

function RoleItem({ role, isSelected, onSelect }: { role: AgentRoleDef; isSelected: boolean; onSelect: () => void }) {
  return (
    <Button
      variant={isSelected ? "secondary" : "ghost"}
      onClick={onSelect}
      style={{ width: "100%", justifyContent: "flex-start", gap: "var(--sp-8)", padding: "8px 12px", fontWeight: isSelected ? 600 : 400 }}
    >
      <span style={badgeStyle(role.color, role.bg)}>{role.letter}</span>
      <span>{role.name}</span>
    </Button>
  );
}

export function KnowledgeRoleSidebar({ selectedRole, onSelectRole }: { selectedRole: string; onSelectRole: (r: string) => void }) {
  return (
    <Card style={{ padding: "var(--sp-16)", width: "240px", flexShrink: 0, height: "fit-content" }}>
      <h3 style={{ margin: "0 0 var(--sp-12) 0", fontSize: "var(--fs-14)", fontWeight: 700, color: "var(--fg)" }}>
        {t("knowledge_agent_roles_title")}
      </h3>
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-8)" }}>
        {KNOWLEDGE_ROLES.map((r) => (
          <RoleItem key={r.id} role={r} isSelected={r.id === selectedRole} onSelect={() => onSelectRole(r.id)} />
        ))}
      </div>
    </Card>
  );
}
