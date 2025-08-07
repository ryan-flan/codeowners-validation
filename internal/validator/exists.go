package validator

import (
	"fmt"
	"runtime"
	"strings"
	"sync"

	"github.com/ryan-flan/codeowners-validation/internal/parser"
)

// ExistsValidator validates that files and directories referenced in CODEOWNERS exist
type ExistsValidator struct {
	RepoPath       string
	SkipDirs       []string
	batchValidator *BatchValidator
}

// Name returns the validator name
func (v *ExistsValidator) Name() string {
	return "exists"
}

// Validate checks if all referenced files/directories exist using optimized batch processing
func (v *ExistsValidator) Validate(rules []parser.CodeOwnerRule) ([]parser.CodeOwnerRule, error) {
	// Initialize batch validator if not already done
	if v.batchValidator == nil {
		v.batchValidator = NewBatchValidatorWithSkipDirs(v.RepoPath, v.SkipDirs)
		if err := v.batchValidator.Initialize(); err != nil {
			return nil, fmt.Errorf("failed to initialize batch validator: %w", err)
		}
	}

	return v.validateRulesBatch(rules)
}

// validateRulesBatch validates rules using high-performance batch processing
func (v *ExistsValidator) validateRulesBatch(rules []parser.CodeOwnerRule) ([]parser.CodeOwnerRule, error) {
	numWorkers := runtime.NumCPU()
	if numWorkers > 12 {
		numWorkers = 12 // Limit for optimal performance
	}

	// Pre-allocate missing slice with estimated capacity
	missing := make([]parser.CodeOwnerRule, 0, len(rules)/10)
	var mu sync.Mutex

	// Split rules into batches for parallel processing
	batchSize := (len(rules) + numWorkers - 1) / numWorkers
	if batchSize < 100 {
		batchSize = 100 // Minimum batch size
	}

	var wg sync.WaitGroup
	errorChan := make(chan error, numWorkers)

	for i := 0; i < len(rules); i += batchSize {
		end := i + batchSize
		if end > len(rules) {
			end = len(rules)
		}

		wg.Add(1)
		go func(batch []parser.CodeOwnerRule) {
			defer wg.Done()

			batchMissing, err := v.validateBatch(batch)
			if err != nil {
				errorChan <- err
				return
			}

			if len(batchMissing) > 0 {
				mu.Lock()
				missing = append(missing, batchMissing...)
				mu.Unlock()
			}
		}(rules[i:end])
	}

	wg.Wait()
	close(errorChan)

	// Check for errors
	if err := <-errorChan; err != nil {
		return nil, err
	}

	return missing, nil
}

// validateBatch validates a batch of rules efficiently
func (v *ExistsValidator) validateBatch(rules []parser.CodeOwnerRule) ([]parser.CodeOwnerRule, error) {
	missing := make([]parser.CodeOwnerRule, 0, len(rules)/10)
	allPaths := v.batchValidator.fsCache.GetAllPaths()

	for _, rule := range rules {
		if !v.ruleHasMatch(rule, allPaths) {
			missing = append(missing, rule)
		}
	}

	return missing, nil
}

// ruleHasMatch efficiently checks if a rule matches any cached path
func (v *ExistsValidator) ruleHasMatch(rule parser.CodeOwnerRule, allPaths []string) bool {
	// Fast path for direct anchored paths
	if strings.HasPrefix(rule.OriginalPath, "/") && !strings.ContainsAny(rule.Pattern, "*?[]") {
		if strings.HasSuffix(rule.OriginalPath, "/") {
			return v.batchValidator.fsCache.DirExists(rule.Pattern)
		}
		return v.batchValidator.fsCache.FileExists(rule.Pattern)
	}

	// Use cached pattern matching for wildcard patterns
	patterns := v.buildGlobPatterns(rule)
	for _, pattern := range patterns {
		for _, path := range allPaths {
			if match, _ := v.batchValidator.patternCache.MatchPath(pattern, path); match {
				return true
			}
		}
	}

	return false
}

// buildGlobPatterns creates glob patterns based on the rule's original path format
func (v *ExistsValidator) buildGlobPatterns(rule parser.CodeOwnerRule) []string {
	pattern := rule.Pattern
	isDirectory := strings.HasSuffix(rule.OriginalPath, "/")
	isAnchored := strings.HasPrefix(rule.OriginalPath, "/")

	var patterns []string

	switch {
	case isAnchored && isDirectory:
		// /docs/ → match "docs" and "docs/**"
		patterns = []string{pattern, pattern + "/**"}
	case isAnchored && !isDirectory:
		// /src/file.rs → match "src/file.rs" exactly
		patterns = []string{pattern}
	case !isAnchored && isDirectory:
		// lib/ → match "**/lib" and "**/lib/**"
		patterns = []string{"**/" + pattern, "**/" + pattern + "/**"}
	case !isAnchored && !isDirectory:
		// *.rs → match "**/*.rs" (or just pattern if it's already a glob)
		if strings.ContainsAny(pattern, "*?[]") {
			patterns = []string{pattern}
		} else {
			patterns = []string{"**/" + pattern}
		}
	}

	return patterns
}
