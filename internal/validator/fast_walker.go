package validator

import (
	"bufio"
	"context"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"slices"
	"strings"
	"sync"
	"sync/atomic"

	"github.com/bmatcuk/doublestar/v4"
)

// Walker provides high-performance concurrent filesystem traversal
type Walker struct {
	rootPath       string
	skipDirs       []string
	gitignoreRules []string
	maxWorkers     int
	bufferSize     int
	pathBuffer     chan string
	resultChan     chan WalkResult
	workerPool     sync.Pool
}

// WalkResult represents a path discovered during traversal
type WalkResult struct {
	Path  string
	IsDir bool
	Error error
}

// NewWalker creates an optimized filesystem walker
func NewWalker(rootPath string, extraSkipDirs []string) *Walker {
	maxWorkers := min(runtime.NumCPU()*2, 16)

	// Default skip directories including .git
	defaultSkipDirs := []string{".git"}
	skipDirs := append(defaultSkipDirs, extraSkipDirs...)

	walker := &Walker{
		rootPath:   rootPath,
		skipDirs:   skipDirs,
		maxWorkers: maxWorkers,
		bufferSize: 10000,
		pathBuffer: make(chan string, 10000),
		resultChan: make(chan WalkResult, 10000),
		workerPool: sync.Pool{
			New: func() any {
				return &pathWorker{}
			},
		},
	}

	// Parse .gitignore if it exists
	walker.parseGitignore()

	return walker
}

// parseGitignore parses the .gitignore file and stores rules
func (w *Walker) parseGitignore() {
	gitignorePath := filepath.Join(w.rootPath, ".gitignore")
	file, err := os.Open(gitignorePath)
	if err != nil {
		// .gitignore doesn't exist or can't be read, skip
		return
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		// Skip empty lines and comments
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		// Remove leading slash if present
		line = strings.TrimPrefix(line, "/")
		w.gitignoreRules = append(w.gitignoreRules, line)
	}
}

// shouldSkipByGitignore checks if a path should be skipped based on .gitignore rules
func (w *Walker) shouldSkipByGitignore(relPath string) bool {
	for _, rule := range w.gitignoreRules {
		// Handle directory rules (ending with /)
		if strings.HasSuffix(rule, "/") {
			dirRule := strings.TrimSuffix(rule, "/")
			if matched, _ := doublestar.Match(dirRule, relPath); matched {
				return true
			}
			// Also check if any parent directory matches
			parts := strings.Split(relPath, "/")
			for i := range parts {
				parentPath := strings.Join(parts[:i+1], "/")
				if matched, _ := doublestar.Match(dirRule, parentPath); matched {
					return true
				}
			}
		} else {
			// Regular file/pattern rule
			if matched, _ := doublestar.Match(rule, relPath); matched {
				return true
			}
			// Check if parent directory matches pattern
			parts := strings.Split(relPath, "/")
			for i := range parts {
				parentPath := strings.Join(parts[:i+1], "/")
				if matched, _ := doublestar.Match(rule, parentPath); matched {
					return true
				}
			}
		}
	}
	return false
}

// pathWorker processes individual paths
type pathWorker struct {
}

// WalkConcurrent performs concurrent filesystem traversal
func (w *Walker) WalkConcurrent(ctx context.Context, callback func(WalkResult)) error {
	var wg sync.WaitGroup
	var processedCount int64

	// Start worker goroutines
	for i := 0; i < w.maxWorkers; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			w.worker(ctx, &processedCount)
		}()
	}

	// Start result processor
	go func() {
		defer close(w.resultChan)
		wg.Wait()
	}()

	// Start directory traversal
	go func() {
		defer close(w.pathBuffer)
		w.traverse(ctx, w.rootPath)
	}()

	// Process results
	for result := range w.resultChan {
		if result.Error == nil && result.Path != "" {
			callback(result)
		}
	}

	return nil
}

// worker processes paths from the buffer
func (w *Walker) worker(ctx context.Context, processedCount *int64) {
	worker := w.workerPool.Get().(*pathWorker)
	defer w.workerPool.Put(worker)

	for {
		select {
		case path, ok := <-w.pathBuffer:
			if !ok {
				return
			}

			result := w.processPath(path)
			select {
			case w.resultChan <- result:
				atomic.AddInt64(processedCount, 1)
			case <-ctx.Done():
				return
			}

		case <-ctx.Done():
			return
		}
	}
}

// processPath processes a single filesystem path
func (w *Walker) processPath(path string) WalkResult {
	stat, err := os.Lstat(path)
	if err != nil {
		return WalkResult{Path: path, Error: err}
	}

	relPath, err := filepath.Rel(w.rootPath, path)
	if err != nil {
		return WalkResult{Path: path, Error: err}
	}

	normalizedRelPath := filepath.ToSlash(relPath)

	// Skip if matches .gitignore rules (this should not happen as we filter in traverse)
	if w.shouldSkipByGitignore(normalizedRelPath) {
		return WalkResult{Path: "", Error: fmt.Errorf("skipped by gitignore"), IsDir: stat.IsDir()}
	}

	return WalkResult{
		Path:  normalizedRelPath,
		IsDir: stat.IsDir(),
	}
}

// traverse recursively traverses directories
func (w *Walker) traverse(ctx context.Context, dirPath string) {
	entries, err := os.ReadDir(dirPath)
	if err != nil {
		return
	}

	for _, entry := range entries {
		select {
		case <-ctx.Done():
			return
		default:
		}

		fullPath := filepath.Join(dirPath, entry.Name())

		// Skip certain directories for performance
		if entry.IsDir() && w.shouldSkipDir(entry.Name()) {
			continue
		}

		// Check if path should be skipped based on .gitignore
		relPath, err := filepath.Rel(w.rootPath, fullPath)
		if err == nil && w.shouldSkipByGitignore(filepath.ToSlash(relPath)) {
			if entry.IsDir() {
				// Skip entire directory tree
				continue
			} else {
				// Skip individual file
				continue
			}
		}

		// Send path to workers with proper channel handling
		select {
		case w.pathBuffer <- fullPath:
		case <-ctx.Done():
			return
		}

		// Recursively traverse subdirectories
		if entry.IsDir() {
			w.traverse(ctx, fullPath) // Remove go routine to avoid channel race
		}
	}
}

// shouldSkipDir checks if a directory should be skipped
func (w *Walker) shouldSkipDir(dirName string) bool {
	if slices.Contains(w.skipDirs, dirName) {
		return true
	}
	return false
}

// SetSkipDirs allows customizing which directories to skip
func (w *Walker) SetSkipDirs(dirs []string) {
	w.skipDirs = dirs
}

// GetStats returns traversal statistics
func (w *Walker) GetStats() (int, int) {
	return w.maxWorkers, w.bufferSize
}
