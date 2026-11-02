import { t } from '../../../core/i18n';

interface ModalProps {
  noteText: string;
  tagsText: string;
  saving: boolean;
  statusMessage: string | null;
  onNoteChange: (val: string) => void;
  onTagsChange: (val: string) => void;
  onSave: () => void;
  onClose: () => void;
}

const NoteFields = ({
  note,
  tags,
  onNote,
  onTags,
}: {
  note: string;
  tags: string;
  onNote: (v: string) => void;
  onTags: (v: string) => void;
}) => (
  <>
    <div style={{ marginBottom: '12px' }}>
      <label style={{ display: 'block', fontSize: '13px', marginBottom: '4px', color: 'var(--mu)' }}>
        {t('tech_profile_dialog_val_label')}
      </label>
      <textarea
        value={note}
        onChange={(e) => onNote(e.target.value)}
        rows={4}
        style={{ width: '100%', padding: '8px', borderRadius: '6px', border: '1px solid var(--ln)', backgroundColor: 'var(--sf2)', color: 'var(--fg)', fontSize: '13px' }}
      />
    </div>
    <div style={{ marginBottom: '16px' }}>
      <label style={{ display: 'block', fontSize: '13px', marginBottom: '4px', color: 'var(--mu)' }}>
        Nhãn
      </label>
      <input
        type="text"
        value={tags}
        onChange={(e) => onTags(e.target.value)}
        style={{ width: '100%', padding: '8px', minHeight: '44px', borderRadius: '6px', border: '1px solid var(--ln)', backgroundColor: 'var(--sf2)', color: 'var(--fg)', fontSize: '13px' }}
      />
    </div>
  </>
);

const NoteActions = ({ saving, onSave, onClose }: { saving: boolean; onSave: () => void; onClose: () => void }) => (
  <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px' }}>
    <button
      type="button"
      onClick={onClose}
      style={{ padding: '6px 12px', minHeight: '44px', border: '1px solid var(--ln)', borderRadius: '8px', backgroundColor: 'var(--sf2)', color: 'var(--fg)' }}
    >
      {t('btn_cancel')}
    </button>
    <button
      type="button"
      onClick={onSave}
      disabled={saving}
      style={{ padding: '6px 16px', minHeight: '44px', border: 'none', borderRadius: '8px', backgroundColor: 'var(--ac)', color: 'var(--acf)', fontWeight: 600 }}
    >
      {saving ? t('loading') : t('run_history_btn_save_note')}
    </button>
  </div>
);

export const NoteEditorModal = ({
  noteText,
  tagsText,
  saving,
  statusMessage,
  onNoteChange,
  onTagsChange,
  onSave,
  onClose,
}: ModalProps) => (
  <div
    role="dialog"
    aria-modal="true"
    style={{ position: 'fixed', inset: 0, backgroundColor: 'rgba(0,0,0,0.5)', display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 1000, padding: '16px' }}
  >
    <div style={{ backgroundColor: 'var(--sf)', border: '1px solid var(--ln)', borderRadius: '10px', width: '100%', maxWidth: '500px', padding: '20px', boxShadow: '0 8px 24px rgba(0,0,0,0.12)' }}>
      <h3 style={{ margin: '0 0 12px 0', fontSize: '16px' }}>{t('run_history_note_label')}</h3>
      <NoteFields note={noteText} tags={tagsText} onNote={onNoteChange} onTags={onTagsChange} />
      {statusMessage && (
        <div style={{ marginBottom: '12px', fontSize: '13px', color: statusMessage.includes('Lỗi') ? 'var(--rd)' : 'var(--gr)' }}>
          {statusMessage}
        </div>
      )}
      <NoteActions saving={saving} onSave={onSave} onClose={onClose} />
    </div>
  </div>
);
