package parser

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/bmatcuk/doublestar/v4"
)

// CodeOwnerRule represents a parsed CODEOWNERS rule
type CodeOwnerRule struct {
	Pattern      string   `json:"pattern"`      // Normalized pattern (no leading/trailing /)
	Owners       []string `json:"owners"`       // List of owners
	OriginalPath string   `json:"originalPath"` // Original path from file (with / if present)
}

// InvalidLine represents a line that couldn't be parsed
type InvalidLine struct {
	LineNumber int    `json:"lineNumber"`
	Content    string `json:"content"`
}

// ParseResult contains the parsing results
type ParseResult struct {
	Rules        []CodeOwnerRule `json:"rules"`
	InvalidLines []InvalidLine   `json:"invalidLines"`
}

// ParseCodeOwnersFile parses a CODEOWNERS file and returns rules and invalid lines
func ParseCodeOwnersFile(filePath string) (*ParseResult, error) {
	file, err := os.Open(filePath)
	if err != nil {
		return nil, fmt.Errorf("failed to open file %s: %w", filePath, err)
	}
	defer file.Close()

	return ParseCodeOwners(file)
}

// ParseCodeOwners parses CODEOWNERS content from a reader
func ParseCodeOwners(reader io.Reader) (*ParseResult, error) {
	// Count non-empty, non-comment lines first for accurate pre-allocation
	ruleCount := countRules(reader)

	// Reset reader to beginning
	if seeker, ok := reader.(io.Seeker); ok {
		seeker.Seek(0, io.SeekStart)
	} else {
		// If reader is not seekable, we'll fall back to default estimation
		if ruleCount == 0 {
			ruleCount = 1000 // Conservative estimate
		}
	}

	// Pre-allocate slices with exact capacity for optimal performance
	rules := make([]CodeOwnerRule, 0, ruleCount)
	invalidLines := make([]InvalidLine, 0, ruleCount/20) // Estimate ~5% invalid lines

	scanner := bufio.NewScanner(reader)
	lineNumber := 0

	for scanner.Scan() {
		lineNumber++
		line := scanner.Text()
		trimmedLine := strings.TrimSpace(line)

		// Skip empty lines and comments
		if trimmedLine == "" || strings.HasPrefix(trimmedLine, "#") {
			continue
		}

		parts := strings.Fields(line)
		if len(parts) == 0 {
			continue
		}

		originalPath := parts[0]
		pattern := strings.Trim(originalPath, "/")
		owners := parts[1:]

		// Basic validation - ensure pattern is not empty after trimming
		if pattern == "" {
			invalidLines = append(invalidLines, InvalidLine{
				LineNumber: lineNumber,
				Content:    line,
			})
			continue
		}

		// Validate the pattern can be used as a valid glob
		if err := validatePattern(pattern, originalPath); err != nil {
			invalidLines = append(invalidLines, InvalidLine{
				LineNumber: lineNumber,
				Content:    line,
			})
			continue
		}

		rule := CodeOwnerRule{
			Pattern:      pattern,
			Owners:       owners,
			OriginalPath: originalPath,
		}

		rules = append(rules, rule)
	}

	if err := scanner.Err(); err != nil {
		return nil, fmt.Errorf("error reading file: %w", err)
	}

	return &ParseResult{
		Rules:        rules,
		InvalidLines: invalidLines,
	}, nil
}

// validatePattern checks if the pattern can be turned into valid globs
func validatePattern(pattern, originalPath string) error {
	isAnchored := strings.HasPrefix(originalPath, "/")
	isDirectory := strings.HasSuffix(originalPath, "/")

	// Test that we can create the necessary globs
	switch {
	case isAnchored && isDirectory:
		// /docs/ → need to create "docs" and "docs/**"
		if !doublestar.ValidatePattern(pattern) {
			return fmt.Errorf("invalid pattern: %s", pattern)
		}
		if !doublestar.ValidatePattern(pattern + "/**") {
			return fmt.Errorf("invalid pattern: %s/**", pattern)
		}
	case isAnchored && !isDirectory:
		// /src/file.rs → need to create "src/file.rs"
		if !doublestar.ValidatePattern(pattern) {
			return fmt.Errorf("invalid pattern: %s", pattern)
		}
	case !isAnchored && isDirectory:
		// lib/ → need to create "**/lib" and "**/lib/**"
		if !doublestar.ValidatePattern("**/" + pattern) {
			return fmt.Errorf("invalid pattern: **/%s", pattern)
		}
		if !doublestar.ValidatePattern("**/" + pattern + "/**") {
			return fmt.Errorf("invalid pattern: **/%s/**", pattern)
		}
	case !isAnchored && !isDirectory:
		// *.rs or file.txt → need to create pattern or "**/pattern"
		if strings.ContainsAny(pattern, "*?[]") {
			// Already a wildcard pattern like *.rs, **/*.md
			if !doublestar.ValidatePattern(pattern) {
				return fmt.Errorf("invalid pattern: %s", pattern)
			}
		} else {
			// Plain file like config.json → match "**/config.json"
			if !doublestar.ValidatePattern("**/" + pattern) {
				return fmt.Errorf("invalid pattern: **/%s", pattern)
			}
		}
	}

	return nil
}

// countRules counts the number of non-empty, non-comment lines for pre-allocation
func countRules(reader io.Reader) int {
	scanner := bufio.NewScanner(reader)
	count := 0

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		// Count potential rules (non-empty, non-comment lines)
		if line != "" && !strings.HasPrefix(line, "#") {
			count++
		}
	}

	return count
}
