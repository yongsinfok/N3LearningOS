import { createBrowserRouter, Navigate } from "react-router-dom";
import Layout from "@/shared/components/Layout";
import DashboardPage from "@/features/dashboard/pages/DashboardPage";
import VocabListPage from "@/features/vocabulary/pages/VocabListPage";
import VocabDetailPage from "@/features/vocabulary/pages/VocabDetailPage";
import VocabQuizPage from "@/features/vocabulary/pages/VocabQuizPage";

export const router = createBrowserRouter([
  {
    element: <Layout />,
    children: [
      { index: true, element: <DashboardPage /> },
      { path: "vocabulary", element: <VocabListPage /> },
      { path: "vocabulary/:id", element: <VocabDetailPage /> },
      { path: "vocabulary/quiz", element: <VocabQuizPage /> },
      { path: "*", element: <Navigate to="/" replace /> },
    ],
  },
]);
