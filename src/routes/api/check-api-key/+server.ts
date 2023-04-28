import { promises as fs } from "fs";
import path from "path";
import type { RequestHandler } from "@sveltejs/kit";

export const GET: RequestHandler = async () => {
  const envPath = path.resolve(".env");

  try {
    const currentContent = await fs.readFile(envPath, "utf8");
    const lines = currentContent.split("\n");
    const apiKeyLineIndex = lines.findIndex(line => line.startsWith("OPENAI_API_KEY="));

    if (apiKeyLineIndex !== -1) {
      return new Response(JSON.stringify({ success: true, message: "API key is present" }), { status: 200 });
    } else {
      return new Response(JSON.stringify({ success: false, message: "API key is not present" }), { status: 404 });
    }
  } catch (error) {
    console.error("Error reading API key from .env:", error);
    return new Response(JSON.stringify({ success: false, message: "Failed to read API key" }), { status: 500 });
  }
};
