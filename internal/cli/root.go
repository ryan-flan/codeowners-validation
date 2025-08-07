package cli

import (
	"fmt"
	"os"
	"strings"

	"github.com/ryan-flan/codeowners-validation/internal/parser"
	"github.com/ryan-flan/codeowners-validation/internal/validator"
	"github.com/spf13/cobra"
)

var (
	checks   string
	path     string
	skipDirs string
)

var rootCmd = &cobra.Command{
	Use:   "codeowners-validation",
	Short: "High-performance CODEOWNERS validator for large repositories",
	Long: `A fast and efficient CODEOWNERS file validator written in Go.
Validates file existence, detects duplicate patterns, and more.
Optimized for large repositories and monorepos.`,
	RunE: runValidation,
}

func init() {
	rootCmd.Flags().StringVar(&checks, "checks", getEnvDefault("INPUT_CHECKS", "all"),
		"Comma-separated list of checks: exists, duplicate_patterns, all")
	rootCmd.Flags().StringVar(&path, "path", ".github/CODEOWNERS",
		"Path to CODEOWNERS file")
	rootCmd.Flags().StringVar(&skipDirs, "skip-dirs", "",
		"Comma-separated list of additional directories to skip during validation")
}

// Execute runs the root command
func Execute() error {
	return rootCmd.Execute()
}

func runValidation(cmd *cobra.Command, args []string) error {
	// Check if CODEOWNERS file exists
	if _, err := os.Stat(path); os.IsNotExist(err) {
		return fmt.Errorf("❌ CODEOWNERS file not found at %q", path)
	}

	// Parse the CODEOWNERS file
	result, err := parser.ParseCodeOwnersFile(path)
	if err != nil {
		return fmt.Errorf("❌ Error parsing CODEOWNERS file: %w", err)
	}

	// Check for invalid lines
	if len(result.InvalidLines) > 0 {
		fmt.Fprintln(os.Stderr, "⚠️  Invalid lines found:")
		for _, line := range result.InvalidLines {
			fmt.Fprintf(os.Stderr, " - Line %d: %s\n", line.LineNumber, line.Content)
		}
		return fmt.Errorf("invalid lines found in the CODEOWNERS file")
	}

	// Create validator arguments
	validatorArgs := validator.NewValidatorArgsFromString(checks)

	// Parse skip directories
	if skipDirs != "" {
		skipDirsList := strings.Split(skipDirs, ",")
		for i := range skipDirsList {
			skipDirsList[i] = strings.TrimSpace(skipDirsList[i])
		}
		validatorArgs.SkipDirs = skipDirsList
	}

	// Run validation
	failedRules, err := validator.RunValidators(validatorArgs, result.Rules)
	if err != nil {
		return fmt.Errorf("❌ Validation failed: %w", err)
	}

	// Report failures
	if len(failedRules) > 0 {
		fmt.Fprintln(os.Stderr, "❌ The following rules failed:")
		for _, failure := range failedRules {
			fmt.Fprintf(os.Stderr, "Validator: %s\n", failure.ValidatorName)
			fmt.Fprintf(os.Stderr, "  Pattern: %s\n", failure.FailedRule.Pattern)
			fmt.Fprintf(os.Stderr, "    Rule: %s\n", failure.FailedRule.OriginalPath)
			fmt.Fprintf(os.Stderr, "    Owners: %v\n", failure.FailedRule.Owners)
			fmt.Fprintln(os.Stderr)
		}
		return fmt.Errorf("some rules failed validation")
	}

	fmt.Println("✅ CODEOWNERS validation passed.")
	return nil
}

// getEnvDefault returns the environment variable value or a default value
func getEnvDefault(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}
