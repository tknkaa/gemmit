/*
Copyright © 2026 NAME HERE <EMAIL ADDRESS>
*/
package cmd

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"strings"

	"github.com/fatih/color"
	"github.com/spf13/cobra"
	"google.golang.org/genai"
)

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "gemmit",
	Short: "Gemini API wrapper for professional-like commit message",
	Long: `A longer description that spans multiple lines and likely contains
examples and usage of using your application. For example:

Cobra is a CLI library for Go that empowers applications.
This application is a tool to generate the needed files
to quickly create a Cobra application.`,
	// Uncomment the following line if your bare application
	// has an action associated with it:
	Run: func(cmd *cobra.Command, args []string) {
		cyan := color.New(color.FgCyan).SprintFunc()
		yellow := color.New(color.FgYellow).SprintFunc()
		red := color.New(color.FgRed).SprintFunc()
		green := color.New(color.FgGreen).SprintFunc()

		fmt.Println(cyan("🤔 Thinking..."))
		ctx := context.Background()
		client, err := genai.NewClient(ctx, nil)
		if err != nil {
			fmt.Println(red("❌ Failed to create Gemini client:"), err)
			return
		}

		// Get list of changed files (including lock files)
		statusCmd := exec.Command("git", "diff", "--cached", "--name-only")
		statusOut, err := statusCmd.Output()
		if err != nil {
			fmt.Println(red("❌ Failed to get git status:"), err)
			return
		}

		if string(statusOut) == "" {
			fmt.Println(yellow("⚠️  No changes are staged"))
			return
		}

		// Check for unwanted directories in staged files
		stagedFiles := string(statusOut)
		warningDirs := []string{"node_modules/", ".direnv/"}
		foundWarningDirs := []string{}
		for _, dir := range warningDirs {
			if strings.Contains(stagedFiles, dir) {
				foundWarningDirs = append(foundWarningDirs, dir)
			}
		}

		if len(foundWarningDirs) > 0 {
			fmt.Println(yellow("⚠️  WARNING: The following directories are in your staged changes:"))
			for _, dir := range foundWarningDirs {
				fmt.Println(red("   - " + dir))
			}
			fmt.Println(yellow("   These directories should typically not be committed!"))
			fmt.Print(yellow("Do you want to continue anyway? [y/N]: "))
			var continueRes string
			fmt.Scanf("%s", &continueRes)
			if continueRes != "y" {
				fmt.Println(yellow("🚫 Commit canceled"))
				return
			}
		}

		// Get diff excluding lock files
		diffCmd := exec.Command("git", "diff", "--cached", "--", ":(exclude)go.sum", ":(exclude)go.mod", ":(exclude)package-lock.json", ":(exclude)yarn.lock", ":(exclude)pnpm-lock.yaml", ":(exclude)bun.lock", ":(exclude)Cargo.lock", ":(exclude)poetry.lock", ":(exclude)uv.lock", ":(exclude)Gemfile.lock")
		diffOut, err := diffCmd.Output()
		if err != nil {
			fmt.Println(red("❌ Failed to get git diff:"), err)
			return
		}

		// Check which lock files were changed
		lockFiles := []string{"go.sum", "go.mod", "package-lock.json", "yarn.lock", "pnpm-lock.yaml", "bun.lock", "Cargo.lock", "poetry.lock", "uv.lock", "Gemfile.lock"}
		changedLockFiles := []string{}
		for _, lockFile := range lockFiles {
			checkCmd := exec.Command("git", "diff", "--cached", "--name-only", "--", lockFile)
			checkOut, _ := checkCmd.Output()
			if string(checkOut) != "" {
				changedLockFiles = append(changedLockFiles, lockFile)
			}
		}

		// Build prompt
		prompt := "Generate a conventional commit message (feat/fix/chore prefix) for this diff. Return only the message, no formatting:\n"
		if len(changedLockFiles) > 0 {
			prompt += "\nNote: The following lock/dependency files were also changed (diff not shown): "
			for i, f := range changedLockFiles {
				if i > 0 {
					prompt += ", "
				}
				prompt += f
			}
			prompt += "\n"
		}
		prompt += "\n" + string(diffOut)

		result, err := client.Models.GenerateContent(
			ctx,
			"gemini-2.5-flash",
			genai.Text(prompt),
			nil,
		)
		if err != nil {
			fmt.Println(red("❌ Gemini API error:"), err)
			return
		}
		generatedCommitMessage := result.Text()
		fmt.Println(cyan("✨ Gemini suggested the following commit message:"))
		fmt.Println(yellow(generatedCommitMessage))
		fmt.Print(green("Do you want to commit with this message? [y/N]: "))
		var res string
		fmt.Scanf("%s", &res)
		if res == "y" {
			commitCommand := exec.Command("git", "commit", "-m", generatedCommitMessage)
			_, err := commitCommand.Output()
			if err != nil {
				fmt.Println(red("❌ Failed to commit:"), err)
				return
			}
			fmt.Println(green("✅ Committed successfully!"))
		} else {
			fmt.Println(yellow("🚫 Commit canceled"))
		}
	},
}

// Execute adds all child commands to the root command and sets flags appropriately.
// This is called by main.main(). It only needs to happen once to the rootCmd.
func Execute() {
	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

func init() {
	// Here you will define your flags and configuration settings.
	// Cobra supports persistent flags, which, if defined here,
	// will be global for your application.

	// rootCmd.PersistentFlags().StringVar(&cfgFile, "config", "", "config file (default is $HOME/.gemmit.yaml)")

	// Cobra also supports local flags, which will only run
	// when this action is called directly.
	// rootCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}
