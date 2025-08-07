package validator

import (
	"fmt"
	"strings"
	"time"

	"github.com/ryan-flan/codeowners-validation/internal/parser"
)

// Validator defines the interface for all validators
type Validator interface {
	Name() string
	Validate(rules []parser.CodeOwnerRule) ([]parser.CodeOwnerRule, error)
}

// ValidatorArgs holds configuration for which validators to run
type ValidatorArgs struct {
	Exists            bool
	DuplicatePatterns bool
	SkipDirs          []string
}

// NewValidatorArgsFromString creates ValidatorArgs from comma-separated string
func NewValidatorArgsFromString(argsStr string) ValidatorArgs {
	args := ValidatorArgs{}

	parts := strings.Split(argsStr, ",")
	for _, part := range parts {
		switch strings.TrimSpace(part) {
		case "exists":
			args.Exists = true
		case "duplicate_patterns":
			args.DuplicatePatterns = true
		case "all":
			args.Exists = true
			args.DuplicatePatterns = true
		}
	}

	return args
}

// ShouldRunAll returns true if no specific validators were requested
func (v ValidatorArgs) ShouldRunAll() bool {
	return !v.Exists && !v.DuplicatePatterns
}

// ValidationResult represents the result of a validation run
type ValidationResult struct {
	ValidatorName string
	FailedRule    parser.CodeOwnerRule
}

// RunValidators runs the specified validators against the rules
func RunValidators(args ValidatorArgs, rules []parser.CodeOwnerRule) ([]ValidationResult, error) {
	var failedRules []ValidationResult

	// Create validator instances
	existsValidator := &ExistsValidator{RepoPath: "."}
	if len(args.SkipDirs) > 0 {
		existsValidator.SkipDirs = args.SkipDirs
	}

	validators := []Validator{
		existsValidator,
		&DuplicatePatternsValidator{},
	}

	for _, validator := range validators {
		shouldRun := args.ShouldRunAll()
		if !shouldRun {
			switch validator.Name() {
			case "exists":
				shouldRun = args.Exists
			case "duplicate_patterns":
				shouldRun = args.DuplicatePatterns
			}
		}

		if shouldRun {
			start := time.Now()
			results, err := validator.Validate(rules)
			duration := time.Since(start)

			if err != nil {
				return nil, fmt.Errorf("validation failed for %s: %w", validator.Name(), err)
			}

			numFailures := len(results)
			for _, rule := range results {
				failedRules = append(failedRules, ValidationResult{
					ValidatorName: validator.Name(),
					FailedRule:    rule,
				})
			}

			fmt.Printf("✓ %s validation completed in %v (%d issues found)\n",
				validator.Name(), duration, numFailures)
		}
	}

	return failedRules, nil
}
