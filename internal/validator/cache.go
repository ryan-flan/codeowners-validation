package validator

import (
	"context"
	"sync"

	"github.com/bmatcuk/doublestar/v4"
)

// FileSystemCache provides cached filesystem operations for massive repos
type FileSystemCache struct {
	repoPath string
	skipDirs []string
	files    map[string]bool
	dirs     map[string]bool
	mu       sync.RWMutex
	once     sync.Once
}

// NewFileSystemCache creates a new filesystem cache
func NewFileSystemCache(repoPath string) *FileSystemCache {
	return &FileSystemCache{
		repoPath: repoPath,
		skipDirs: nil,
		files:    make(map[string]bool, 100000), // Pre-allocate for large repos
		dirs:     make(map[string]bool, 10000),
	}
}

// NewFileSystemCacheWithSkipDirs creates a new filesystem cache with custom skip directories
func NewFileSystemCacheWithSkipDirs(repoPath string, skipDirs []string) *FileSystemCache {
	return &FileSystemCache{
		repoPath: repoPath,
		skipDirs: skipDirs,
		files:    make(map[string]bool, 100000), // Pre-allocate for large repos
		dirs:     make(map[string]bool, 10000),
	}
}

// BuildCache builds the complete filesystem cache in one pass
func (c *FileSystemCache) BuildCache() error {
	var buildErr error
	c.once.Do(func() {
		buildErr = c.buildCacheOnce()
	})
	return buildErr
}

func (c *FileSystemCache) buildCacheOnce() error {
	walker := NewWalker(c.repoPath, c.skipDirs) // Use configured skip dirs
	ctx := context.Background()

	c.mu.Lock()
	defer c.mu.Unlock()

	return walker.WalkConcurrent(ctx, func(result WalkResult) {
		if result.IsDir {
			c.dirs[result.Path] = true
		} else {
			c.files[result.Path] = true
		}
	})
}

// FileExists checks if a file exists in the cache
func (c *FileSystemCache) FileExists(path string) bool {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.files[path]
}

// DirExists checks if a directory exists in the cache
func (c *FileSystemCache) DirExists(path string) bool {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.dirs[path]
}

// GetAllPaths returns all cached paths for pattern matching
func (c *FileSystemCache) GetAllPaths() []string {
	c.mu.RLock()
	defer c.mu.RUnlock()

	// Combine files and directories
	paths := make([]string, 0, len(c.files)+len(c.dirs))

	for path := range c.files {
		paths = append(paths, path)
	}

	for path := range c.dirs {
		paths = append(paths, path)
	}

	return paths
}

// PatternCache provides compiled pattern caching for glob operations
type PatternCache struct {
	patterns map[string]bool
}

// NewPatternCache creates a new pattern cache
func NewPatternCache() *PatternCache {
	return &PatternCache{
		patterns: make(map[string]bool, 1000),
	}
}

// MatchPath efficiently matches a path against a pattern using the cache
func (c *PatternCache) MatchPath(pattern, path string) (bool, error) {
	// Use doublestar.Match directly for now - can be optimized further if needed
	return doublestar.Match(pattern, path)
}

// BatchValidator provides high-performance validation for large rule sets
type BatchValidator struct {
	fsCache      *FileSystemCache
	patternCache *PatternCache
	batchSize    int
}

// NewBatchValidator creates a new batch validator optimized for large repos
func NewBatchValidator(repoPath string) *BatchValidator {
	return &BatchValidator{
		fsCache:      NewFileSystemCache(repoPath),
		patternCache: NewPatternCache(),
		batchSize:    1000, // Process rules in batches of 1000
	}
}

// NewBatchValidatorWithSkipDirs creates a new batch validator with custom skip directories
func NewBatchValidatorWithSkipDirs(repoPath string, skipDirs []string) *BatchValidator {
	return &BatchValidator{
		fsCache:      NewFileSystemCacheWithSkipDirs(repoPath, skipDirs),
		patternCache: NewPatternCache(),
		batchSize:    1000, // Process rules in batches of 1000
	}
}

// Initialize prepares the validator by building caches
func (bv *BatchValidator) Initialize() error {
	return bv.fsCache.BuildCache()
}
