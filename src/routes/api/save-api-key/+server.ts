import { promises as fs } from "fs";
import path from "path";
import type { RequestHandler } from "@sveltejs/kit";

export const POST: RequestHandler = async ({ request }) => {
    const body = await request.arrayBuffer();
    const { apiKey } = JSON.parse(new TextDecoder().decode(body));

    if (!apiKey) {
        return new Response(JSON.stringify({ success: false, message: "API key is required" }), { status: 400 });
    }

    const envPath = path.resolve(".env");

    try {
      const currentContent = await fs.readFile(envPath, "utf8");
      const lines = currentContent.split("\n");
      const apiKeyLineIndex = lines.findIndex(line => line.startsWith("OPENAI_API_KEY="));

      if (apiKeyLineIndex !== -1) {
        lines[apiKeyLineIndex] = `OPENAI_API_KEY=${apiKey}`;
      } else {
        lines.push(`OPENAI_API_KEY=${apiKey}`);
      }

      const updatedContent = lines.join("\n");

      await fs.writeFile(envPath, updatedContent);
      return new Response(JSON.stringify({ success: true, message: "API key saved successfully" }), { status: 200 });
    } catch (error) {
      console.error("Error updating API key in .env:", error);
      return new Response(JSON.stringify({ success: false, message: "Failed to save API key" }), { status: 500 });
    }
};
