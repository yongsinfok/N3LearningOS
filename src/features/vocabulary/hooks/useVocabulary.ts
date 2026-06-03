import { useQuery } from "@tanstack/react-query";
import {
  listVocabulary,
  getVocabularyDetail,
  getDueFlashcards,
  reviewFlashcard,
} from "@/services/api";

export function useVocabList(page = 1, search?: string) {
  return useQuery({
    queryKey: ["vocabulary", "list", page, search],
    queryFn: () => listVocabulary(page, search, 20),
  });
}

export function useVocabDetail(id: string) {
  return useQuery({
    queryKey: ["vocabulary", id],
    queryFn: () => getVocabularyDetail(id),
    enabled: !!id,
  });
}

export function useDueFlashcards(contentType?: string, limit?: number) {
  return useQuery({
    queryKey: ["flashcards", "due", contentType, limit],
    queryFn: () => getDueFlashcards(contentType, limit),
  });
}

export function useReviewFlashcard() {
  return async (cardId: string, rating: number, durationSecs?: number) => {
    return reviewFlashcard(cardId, rating, durationSecs);
  };
}
