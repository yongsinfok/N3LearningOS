import { useMutation } from "@tanstack/react-query";
import {
  importVocabularyFromFile,
  importGrammarFromFile,
  importKanjiFromFile,
  importReadingFromFile,
  importListeningFromFile,
  ImportFileResult,
} from "@/services/api";
import { ImportContentType } from "../types";

const importFns: Record<ImportContentType, (filePath: string, mode: "Append" | "Overwrite") => Promise<ImportFileResult>> = {
  vocabulary: importVocabularyFromFile,
  grammar: importGrammarFromFile,
  kanji: importKanjiFromFile,
  reading: importReadingFromFile,
  listening: importListeningFromFile,
};

export function useImport(contentType: ImportContentType) {
  return useMutation({
    mutationFn: ({
      filePath,
      mode,
    }: {
      filePath: string;
      mode: "Append" | "Overwrite";
    }) => importFns[contentType](filePath, mode),
  });
}
