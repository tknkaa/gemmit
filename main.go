package main

import (
	"context"
	"fmt"
	"os"
	"os/exec"

	"google.golang.org/genai"
)

func main() {
	cmd := exec.Command("git", "diff", "--staged")
	diff, err := cmd.Output()
	if len(diff) == 0 {
		fmt.Println("Error: You don't have any staged changes. Please stage your changes before committing.")
		return
	}
	fmt.Println(string(diff))
	if err != nil {
		fmt.Printf("Error: %s\n", err)
		return
	}

	apiKey := os.Getenv("GEMINI_API_KEY")
	if apiKey == "" {
		fmt.Println("Error: Please set the GEMINI_API_KEY environment variable.")
		return
	}

	ctx := context.Background()
	client, err := genai.NewClient(ctx, &genai.ClientConfig{
		APIKey: apiKey,
	},
	)

	if err != nil {
		fmt.Printf("Error creating Gemini client: %s\n", err)
		return
	}

	result, err := client.Models.GenerateContent(
		ctx,
		"gemini-2.5-flash",
		genai.Text("Explain how AI works"),
		nil,
	)

	if err != nil {
		fmt.Printf("Error generating content: %s\n", err)
		return
	}
	fmt.Println(result.Text())
}
