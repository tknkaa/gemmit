package cmd

import (
	"context"
	"fmt"
	"os/exec"
	"strings"

	"github.com/charmbracelet/bubbles/spinner"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"google.golang.org/genai"
)

var (
	titleStyle   = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("99"))
	errorStyle   = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("196"))
	warningStyle = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("214"))
	successStyle = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("42"))
	messageStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("229"))
	promptStyle  = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("86"))
)

type state int

const (
	stateLoading state = iota
	stateWarning
	stateConfirm
	stateCommitting
	stateDone
	stateError
)

type model struct {
	state              state
	spinner            spinner.Model
	err                error
	commitMessage      string
	warningDirs        []string
	ctx                context.Context
	client             *genai.Client
	prompt             string
	stagedFiles        string
	changedLockFiles   []string
}

type generatedMsg struct {
	message string
	err     error
}

type commitDoneMsg struct {
	err error
}

func initialModel() model {
	s := spinner.New()
	s.Spinner = spinner.Dot
	s.Style = lipgloss.NewStyle().Foreground(lipgloss.Color("205"))

	return model{
		state:   stateLoading,
		spinner: s,
		ctx:     context.Background(),
	}
}

func (m model) Init() tea.Cmd {
	return tea.Batch(m.spinner.Tick, m.checkGitAndGenerate())
}

func (m model) checkGitAndGenerate() tea.Cmd {
	return func() tea.Msg {
		// Create Gemini client
		client, err := genai.NewClient(m.ctx, nil)
		if err != nil {
			return generatedMsg{err: fmt.Errorf("failed to create Gemini client: %w", err)}
		}

		// Get list of changed files
		statusCmd := exec.Command("git", "diff", "--cached", "--name-only")
		statusOut, err := statusCmd.Output()
		if err != nil {
			return generatedMsg{err: fmt.Errorf("failed to get git status: %w", err)}
		}

		if string(statusOut) == "" {
			return generatedMsg{err: fmt.Errorf("no changes are staged")}
		}

		stagedFiles := string(statusOut)

		// Check for unwanted directories
		warningDirs := []string{"node_modules/", ".direnv/"}
		foundWarningDirs := []string{}
		for _, dir := range warningDirs {
			if strings.Contains(stagedFiles, dir) {
				foundWarningDirs = append(foundWarningDirs, dir)
			}
		}

		// Get diff excluding lock files
		diffCmd := exec.Command("git", "diff", "--cached", "--", ":(exclude)go.sum", ":(exclude)go.mod", ":(exclude)package-lock.json", ":(exclude)yarn.lock", ":(exclude)pnpm-lock.yaml", ":(exclude)bun.lock", ":(exclude)Cargo.lock", ":(exclude)poetry.lock", ":(exclude)uv.lock", ":(exclude)Gemfile.lock")
		diffOut, err := diffCmd.Output()
		if err != nil {
			return generatedMsg{err: fmt.Errorf("failed to get git diff: %w", err)}
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
		prompt := "Generate a concise conventional commit message (feat/fix/chore prefix) for this diff. Keep it short and to the point - ideally one line. Return only the commit message, no explanations or formatting:\n"
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

		// If there are warnings, return early to show warning state
		if len(foundWarningDirs) > 0 {
			return generatedMsg{
				message: "",
				err:     fmt.Errorf("WARNING_DIRS:%s", strings.Join(foundWarningDirs, ",")),
			}
		}

		// Generate commit message
		result, err := client.Models.GenerateContent(
			context.Background(),
			"gemini-2.5-flash",
			genai.Text(prompt),
			nil,
		)
		if err != nil {
			return generatedMsg{err: fmt.Errorf("Gemini API error: %w", err)}
		}

		return generatedMsg{message: result.Text()}
	}
}

func (m model) generateCommitMessage() tea.Cmd {
	return func() tea.Msg {
		client, err := genai.NewClient(m.ctx, nil)
		if err != nil {
			return generatedMsg{err: fmt.Errorf("failed to create Gemini client: %w", err)}
		}

		result, err := client.Models.GenerateContent(
			context.Background(),
			"gemini-2.5-flash",
			genai.Text(m.prompt),
			nil,
		)
		if err != nil {
			return generatedMsg{err: fmt.Errorf("Gemini API error: %w", err)}
		}

		return generatedMsg{message: result.Text()}
	}
}

func (m model) commitChanges() tea.Cmd {
	return func() tea.Msg {
		commitCommand := exec.Command("git", "commit", "-m", m.commitMessage)
		_, err := commitCommand.Output()
		return commitDoneMsg{err: err}
	}
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch m.state {
		case stateWarning:
			switch msg.String() {
			case "y", "Y":
				m.state = stateLoading
				return m, tea.Batch(m.spinner.Tick, m.generateCommitMessage())
			case "n", "N", "q", "ctrl+c":
				m.state = stateDone
				m.err = fmt.Errorf("commit canceled")
				return m, tea.Quit
			}

		case stateConfirm:
			switch msg.String() {
			case "y", "Y":
				m.state = stateCommitting
				return m, tea.Batch(m.spinner.Tick, m.commitChanges())
			case "n", "N", "q", "ctrl+c":
				m.state = stateDone
				m.err = fmt.Errorf("commit canceled")
				return m, tea.Quit
			}

		case stateDone, stateError:
			return m, tea.Quit
		}

	case generatedMsg:
		if msg.err != nil {
			// Check if it's a warning
			if strings.HasPrefix(msg.err.Error(), "WARNING_DIRS:") {
				m.warningDirs = strings.Split(strings.TrimPrefix(msg.err.Error(), "WARNING_DIRS:"), ",")
				m.state = stateWarning
				return m, nil
			}
			m.state = stateError
			m.err = msg.err
			return m, tea.Quit
		}
		m.commitMessage = msg.message
		m.state = stateConfirm
		return m, nil

	case commitDoneMsg:
		m.state = stateDone
		if msg.err != nil {
			m.err = fmt.Errorf("failed to commit: %w", msg.err)
		}
		return m, tea.Quit

	case spinner.TickMsg:
		var cmd tea.Cmd
		m.spinner, cmd = m.spinner.Update(msg)
		return m, cmd
	}

	return m, nil
}

func (m model) View() string {
	var s strings.Builder

	switch m.state {
	case stateLoading:
		s.WriteString(fmt.Sprintf("\n %s %s\n\n", m.spinner.View(), titleStyle.Render("Thinking...")))

	case stateWarning:
		s.WriteString("\n" + warningStyle.Render("⚠️  WARNING") + "\n\n")
		s.WriteString(warningStyle.Render("The following directories are in your staged changes:") + "\n")
		for _, dir := range m.warningDirs {
			s.WriteString(errorStyle.Render("  • "+dir) + "\n")
		}
		s.WriteString("\n" + warningStyle.Render("These directories should typically not be committed!") + "\n\n")
		s.WriteString(promptStyle.Render("Continue anyway? (y/N): "))

	case stateConfirm:
		s.WriteString("\n" + titleStyle.Render("✨ Gemini suggested:") + "\n\n")
		s.WriteString(messageStyle.Render(m.commitMessage) + "\n\n")
		s.WriteString(promptStyle.Render("Commit with this message? (y/N): "))

	case stateCommitting:
		s.WriteString(fmt.Sprintf("\n %s %s\n\n", m.spinner.View(), titleStyle.Render("Committing...")))

	case stateDone:
		if m.err != nil {
			s.WriteString("\n" + warningStyle.Render("🚫 "+m.err.Error()) + "\n\n")
		} else {
			s.WriteString("\n" + successStyle.Render("✅ Committed successfully!") + "\n\n")
		}

	case stateError:
		s.WriteString("\n" + errorStyle.Render("❌ Error: "+m.err.Error()) + "\n\n")
	}

	return s.String()
}

func runTUI() error {
	p := tea.NewProgram(initialModel())
	_, err := p.Run()
	return err
}
