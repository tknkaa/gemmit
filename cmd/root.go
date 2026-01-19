/*
Copyright © 2026 NAME HERE <EMAIL ADDRESS>
*/
package cmd

import (
	"context"
	"fmt"
	"os"
	"os/exec"

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
		ctx := context.Background()
		client, err := genai.NewClient(ctx, nil)
		if err != nil {
			fmt.Println("Failed to create Gemini client:", err)
			return
		}

		diffCmd := exec.Command("git", "diff", "--cached")
		diffOut, err := diffCmd.Output()
		if err != nil {
			fmt.Println("Failed to get git diff:", err)
			return
		}

		if string(diffOut) == "" {
			fmt.Println("No changes are staged")
			return
		}

		prompt := "Generate a professional, clear, and concise commit message for the following git diff:\n" + string(diffOut)

		result, err := client.Models.GenerateContent(
			ctx,
			"gemini-3-flash-preview",
			genai.Text(prompt),
			nil,
		)
		if err != nil {
			fmt.Println("Gemini API error:", err)
			return
		}
		generatedCommitMessage := result.Text()
		fmt.Println("Gemini suggested the following commit message")
		fmt.Println(generatedCommitMessage)
		fmt.Println("Do you want to commit with this message?[y/N]")
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
