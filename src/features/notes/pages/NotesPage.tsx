import { useNotes, useDeleteNote } from "../hooks/useNotes";

export default function NotesPage() {
  const { data: notes, isLoading } = useNotes();
  const deleteNote = useDeleteNote();

  if (isLoading) {
    return <p className="text-muted-foreground">加载中...</p>;
  }

  if (!notes || notes.length === 0) {
    return (
      <div className="space-y-4">
        <h1 className="text-2xl font-bold">笔记</h1>
        <p className="text-muted-foreground">
          还没有笔记。在学习内容页面可以添加笔记。
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">笔记</h1>
      <div className="space-y-3">
        {notes.map((note) => (
          <div key={note.id} className="rounded-lg border p-4">
            <div className="mb-2 flex items-center justify-between">
              <span className="text-xs text-muted-foreground">
                {note.content_type} · {note.updated_at.slice(0, 10)}
              </span>
              <button
                onClick={() => deleteNote.mutate(note.id)}
                className="text-xs text-destructive hover:underline"
              >
                删除
              </button>
            </div>
            <p className="whitespace-pre-wrap text-sm">{note.content}</p>
          </div>
        ))}
      </div>
    </div>
  );
}
