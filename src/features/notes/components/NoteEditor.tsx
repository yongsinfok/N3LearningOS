import { useState, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { useSaveNote } from "../hooks/useNotes";

interface NoteEditorProps {
  contentId: string;
  contentType: string;
  initialContent?: string;
}

export default function NoteEditor({ contentId, contentType, initialContent }: NoteEditorProps) {
  const [content, setContent] = useState(initialContent || "");
  const saveNote = useSaveNote();

  useEffect(() => {
    setContent(initialContent || "");
  }, [initialContent]);

  const handleSave = () => {
    saveNote.mutate({ contentId, contentType, content });
  };

  return (
    <div className="space-y-2">
      <Textarea
        value={content}
        onChange={(e) => setContent(e.target.value)}
        placeholder="添加笔记..."
        rows={4}
      />
      <Button
        size="sm"
        onClick={handleSave}
        disabled={saveNote.isPending || !content.trim()}
      >
        {saveNote.isPending ? "保存中..." : "保存笔记"}
      </Button>
    </div>
  );
}
