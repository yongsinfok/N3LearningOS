import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { listNotes, saveNote, deleteNote } from "@/services/api";

export function useNotes(contentId?: string, contentType?: string) {
  return useQuery({
    queryKey: ["notes", contentId, contentType],
    queryFn: () => listNotes(contentId, contentType),
  });
}

export function useSaveNote() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({
      contentId,
      contentType,
      content,
    }: {
      contentId: string;
      contentType: string;
      content: string;
    }) => saveNote(contentId, contentType, content),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["notes"] }),
  });
}

export function useDeleteNote() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => deleteNote(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["notes"] }),
  });
}
