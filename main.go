package main

import (
	"context"
	"fmt"
	"os"
	"os/exec"

	"google.golang.org/genai"
)

var lockfiles = [7]string{"package-lock.json", "pnpm-lock.yaml", "bun.lock", "uv.lock", "go.sum", "Cargo.lock", "flake.lock"}

func main() {
	ignoredFiles := []string{}

	for _, lockfile := range lockfiles {
		ignoredFiles = append(ignoredFiles, fmt.Sprintf(":!%s", lockfile))
	}

	args := append([]string{"diff", "--staged", "--"}, ignoredFiles...)

	cmd := exec.Command("git", args...)

	diff, err := cmd.Output()
	if len(diff) == 0 {
		fmt.Println("Error: You don't have any staged changes. Please stage your changes before committing.")
		return
	}

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
		genai.Text(fmt.Sprintf("Here is a git diff of my staged changes:\n\n%s\n\nPlease provide a concise and clear commit message that accurately describes the changes made in this diff.", string(diff))),
		nil,
	)

	if err != nil {
		fmt.Printf("Error generating content: %s\n", err)
		return
	}

	message := result.Text()

	fmt.Printf("Suggested commit message:\n%s\n", message)
	fmt.Print("Do you want to use this commit message? (y/n): ")
	var response string
	fmt.Scanln(&response)
	if response == "y" || response == "Y" {
		cmd := exec.Command("git", "commit", "-m", message)
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		err := cmd.Run()
		if err != nil {
			fmt.Printf("Error committing changes: %s\n", err)
			return
		}
		fmt.Println("Changes committed successfully.")
	} else {
		fmt.Println("Commit message discarded. Please edit the message and commit manually.")
	}
}
