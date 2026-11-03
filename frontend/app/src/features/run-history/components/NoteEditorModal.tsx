import { t } from '../../../core/i18n';
import { Modal } from '../../../ui/components/Modal';
import { Button } from '../../../ui/components/Button';
import { Input } from '../../../ui/components/Input';
import { Textarea } from '../../../ui/components/Textarea';

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
      <Textarea
        value={note}
        onChange={(e) => onNote(e.target.value)}
        rows={4}
      />
    </div>
    <div style={{ marginBottom: '16px' }}>
      <label style={{ display: 'block', fontSize: '13px', marginBottom: '4px', color: 'var(--mu)' }}>{t("note_label")}</label>
      <Input
        type="text"
        value={tags}
        onChange={(e) => onTags(e.target.value)}
      />
    </div>
  </>
);

const NoteActions = ({ saving, onSave, onClose }: { saving: boolean; onSave: () => void; onClose: () => void }) => (
  <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px' }}>
    <Button
      type="button"
      variant="secondary"
      onClick={onClose}
    >
      {t('btn_cancel')}
    </Button>
    <Button
      type="submit"
      variant="primary"
      onClick={onSave}
      disabled={saving}
    >
      {saving ? t('loading') : t('run_history_btn_save_note')}
    </Button>
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
}: ModalProps) => {
  const footer = <NoteActions saving={saving} onSave={onSave} onClose={onClose} />;

  return (
    <Modal open onClose={onClose} title={t('run_history_note_label')} footer={footer} size="md">
      <NoteFields note={noteText} tags={tagsText} onNote={onNoteChange} onTags={onTagsChange} />
      {statusMessage && (
        <div style={{ marginBottom: '12px', fontSize: '13px', color: statusMessage.includes('Lỗi') ? 'var(--rd)' : 'var(--gr)' }}>
          {statusMessage}
        </div>
      )}
    </Modal>
  );
};
