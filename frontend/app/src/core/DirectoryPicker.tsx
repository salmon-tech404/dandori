import { FC, useCallback, useEffect, useState } from "react";
import { ArrowUp, Folder, GitBranch } from "lucide-react";
import { t } from "./i18n";
import { apiClient, getSafeErrorMessage } from "./api";
import { Button } from "../ui/components/Button";
import { Modal } from "../ui/components/Modal";

interface DirEntry {
  name: string;
  path: string;
  is_git: boolean;
}

interface DirListing {
  path: string;
  parent: string | null;
  is_git: boolean;
  entries: DirEntry[];
  truncated: boolean;
}

export interface DirectoryPickerProps {
  /** Thư mục bắt đầu (bỏ trống = danh sách ổ đĩa). */
  initialPath?: string;
  onSelect: (path: string) => void;
  onClose: () => void;
}

function useDirListing(initialPath: string) {
  const [path, setPath] = useState(initialPath);
  const [listing, setListing] = useState<DirListing | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async (target: string) => {
    setLoading(true);
    setError(null);
    try {
      const res = await apiClient<DirListing>(`/api/v1/fs/dirs?path=${encodeURIComponent(target)}`);
      setListing(res);
      setPath(res.path);
    } catch (err) {
      const safe = getSafeErrorMessage(err);
      setError(safe.code === "forbidden" ? t("picker_forbidden") : safe.userMessage);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load(initialPath);
  }, [load, initialPath]);

  return { path, listing, error, loading, load };
}

function EntryRow({ entry, onOpen }: { entry: DirEntry; onOpen: (p: string) => void }) {
  return (
    <li>
      <Button
        variant="ghost"
        type="button"
        onClick={() => onOpen(entry.path)}
        style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)", width: "100%", minHeight: "var(--menu-height)", height: "auto", padding: "0 var(--sp-12)", background: "none", border: "none", borderRadius: "var(--radius-8)", color: "var(--fg)", fontSize: "var(--fs-14)", textAlign: "left" }}
      >
        <Folder size={16} />
        <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{entry.name}</span>
        {entry.is_git && (
          <span style={{ display: "inline-flex", alignItems: "center", gap: "4px", fontSize: "var(--fs-12)", color: "var(--gr)", backgroundColor: "var(--grb)", borderRadius: "var(--radius-pill)", padding: "0 8px" }}>
            <GitBranch size={12} />
            {t("picker_git_badge")}
          </span>
        )}
      </Button>
    </li>
  );
}

function PickerList({ listing, loading, onOpen }: { listing: DirListing | null; loading: boolean; onOpen: (p: string) => void }) {
  if (loading) return <p style={{ color: "var(--mu)", fontSize: "var(--fs-13)" }}>{t("picker_loading")}</p>;
  if (!listing || listing.entries.length === 0) return <p style={{ color: "var(--mu)", fontSize: "var(--fs-13)" }}>{t("picker_empty")}</p>;
  return (
    <ul style={{ listStyle: "none", margin: 0, padding: 0, display: "flex", flexDirection: "column", gap: "2px" }}>
      {listing.entries.map((e) => <EntryRow key={e.path} entry={e} onOpen={onOpen} />)}
    </ul>
  );
}

/** Hộp thoại chọn thư mục kho Git trên máy chủ (trình duyệt không cho biết đường dẫn thật, nên duyệt qua máy chủ). */
export const DirectoryPicker: FC<DirectoryPickerProps> = ({ initialPath = "", onSelect, onClose }) => {
  const { path, listing, error, loading, load } = useDirListing(initialPath);
  const canUse = Boolean(listing?.is_git) && !loading;

  const footer = (
    <div style={{ display: "flex", justifyContent: "flex-end", gap: "var(--sp-8)" }}>
      <Button type="button" variant="ghost" onClick={onClose}>{t("btn_cancel")}</Button>
      <Button type="button" variant="primary" disabled={!canUse} onClick={() => listing && onSelect(listing.path)}>{t("picker_use_this")}</Button>
    </div>
  );

  return (
    <Modal open onClose={onClose} title={t("picker_title")} footer={footer} size="md">
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-12)" }}>
        <div style={{ display: "flex", alignItems: "center", gap: "var(--sp-8)" }}>
          <Button type="button" variant="ghost" size="icon" aria-label={t("picker_up")} disabled={!listing?.path} onClick={() => void load(listing?.parent ?? "")} icon={<ArrowUp size={16} />} />
          <code data-testid="picker-path" style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", fontSize: "var(--fs-13)" }}>{path || t("picker_roots")}</code>
        </div>
        {error && <div role="alert" style={{ color: "var(--rd)", fontSize: "var(--fs-13)" }}>{error}</div>}
        <div style={{ minHeight: "160px", maxHeight: "40vh", overflowY: "auto" }}>
          <PickerList listing={listing} loading={loading} onOpen={(p) => void load(p)} />
        </div>
        {listing?.truncated && <small style={{ color: "var(--mu)" }}>{t("picker_truncated")}</small>}
        {listing?.path && <small style={{ color: listing.is_git ? "var(--gr)" : "var(--mu)" }}>{listing.is_git ? t("picker_is_git") : t("picker_not_git")}</small>}
      </div>
    </Modal>
  );
};
