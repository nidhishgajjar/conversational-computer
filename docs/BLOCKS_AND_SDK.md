# Blocks & SDK Design

## Block Philosophy

Blocks are **conversation artifacts**, not windows or apps. They appear in response to user intent, can be interacted with if needed, but primarily serve the conversation flow.

## Block Types

### 1. Native Blocks (Maximum Performance)

Built with our stack, fully controlled:

```rust
struct NativeBlock {
    renderer: SkiaCanvas,
    state: BlockState,
    transitions: AnimationEngine,
}
```

**Examples:**
- **TextBlock** - Conversation display
- **CodeBlock** - Syntax highlighted code with editing
- **MediaBlock** - Video/audio player
- **SystemBlock** - System status, settings
- **TerminalBlock** - Command execution

### 2. Web Blocks (Maximum Compatibility)

Embedded web views with AI control:

```rust
struct WebBlock {
    webview: WebKitGTK,
    injection: JSController,
    masking: UIHider,
}
```

**Examples:**
- **SocialBlock** - X, Instagram, Reddit
- **StreamingBlock** - Netflix, YouTube, Spotify
- **ProductivityBlock** - Google Docs, Notion
- **GenericWebBlock** - Any website

**AI Control via JS Injection:**
```javascript
// Remove UI chrome for cleaner display
document.querySelector('.sidebar').style.display = 'none';
document.querySelector('.header').style.display = 'none';

// AI operates the page
document.querySelector('#search').value = 'Elon Musk';
document.querySelector('#search-button').click();
```

### 3. Hybrid Blocks (Best of Both)

Native UI with service APIs:

```rust
struct HybridBlock {
    ui: NativeRenderer,      // Our UI
    api: ServiceClient,       // Their API
    cache: LocalStorage,      // Offline support
}
```

**Examples:**

#### Slack Block
```rust
struct SlackBlock {
    ui: NativeUI,           // Custom Slack-styled UI
    api: SlackAPI,          // Official API
    messages: Vec<Message>,
}

// User: "Show #general from last week"
// SPOC: Creates SlackBlock, fetches via API, renders natively
```

#### Spotify Block
```rust
struct SpotifyBlock {
    ui: MusicPlayerUI,      // Native player controls
    api: SpotifyWebAPI,     // Official API
    audio: GStreamer,       // Direct audio playback
}
```

## Core Blocks We Need

### Essential (Day 1)
1. **ConversationBlock** - Display chat history
2. **InputBlock** - User input (already have)
3. **WebBlock** - Basic web browsing
4. **CodeBlock** - Code viewing/editing
5. **MediaBlock** - Images/video display

### Priority (Week 1)
6. **TerminalBlock** - Run commands
7. **FileBlock** - File browser/manager
8. **SettingsBlock** - System settings
9. **NotificationBlock** - System alerts
10. **SearchBlock** - Universal search

### Extended (Month 1)
11. **EmailBlock** - Email client
12. **CalendarBlock** - Schedule management
13. **NotesBlock** - Note taking
14. **MusicBlock** - Music player
15. **WeatherBlock** - Weather info

## SDK Design

### For Block Developers

```rust
// Simple trait-based API
#[block_metadata(
    name = "Twitter Block",
    version = "1.0.0",
    author = "AIOS Team",
    capabilities = ["network", "storage"],
    intents = ["show tweets", "post tweet"]
)]
pub struct TwitterBlock {
    state: TwitterState,
}

impl Block for TwitterBlock {
    fn init(&mut self, config: BlockConfig) -> Result<()> {
        // Initialize block
    }
    
    fn render(&mut self, canvas: &mut Canvas) -> Result<()> {
        // Draw to canvas
    }
    
    fn handle_input(&mut self, event: InputEvent) -> BlockResponse {
        // Handle keyboard/mouse when focused
    }
    
    fn handle_intent(&mut self, intent: Intent) -> Result<()> {
        // Handle AI commands
        match intent {
            Intent::ShowTweets { user, timeframe } => {
                self.fetch_tweets(user, timeframe)?;
            }
            Intent::PostTweet { content } => {
                self.post_tweet(content)?;
            }
        }
    }
}
```

### Block Lifecycle

```rust
enum BlockState {
    // Lifecycle states
    Initializing,       // Being created
    Active,            // Visible and running
    Background,        // Running but not visible
    Suspended,         // Saved to disk
    Destroying,        // Being removed
}

trait BlockLifecycle {
    fn on_create(&mut self);
    fn on_show(&mut self);
    fn on_hide(&mut self);
    fn on_focus(&mut self);
    fn on_blur(&mut self);
    fn on_destroy(&mut self);
}
```

### Block Communication

```rust
// Blocks can communicate via SPOC
enum BlockMessage {
    // Inter-block communication
    SendData { to: BlockId, data: Value },
    RequestData { from: BlockId, query: Query },
    
    // System events
    SystemEvent(SystemEvent),
    
    // AI orchestration
    AICommand(Intent),
}

// Example: CodeBlock → TerminalBlock
code_block.send(BlockMessage::SendData {
    to: terminal_block_id,
    data: json!({ "command": "cargo run" })
});
```

## Transition System

Every block appearance/disappearance is smooth:

```rust
pub struct TransitionEngine {
    animations: Vec<Animation>,
}

enum Animation {
    // Entrance animations
    SlideUp { duration: f32, ease: EaseFunction },
    FadeIn { duration: f32 },
    Expand { from: Rect, to: Rect, duration: f32 },
    
    // Exit animations
    SlideDown { duration: f32 },
    FadeOut { duration: f32 },
    Collapse { duration: f32 },
    
    // Focus animations
    Glow { intensity: f32 },
    Scale { factor: f32 },
    Blur { radius: f32 },
}

// Example flow
fn show_web_block(&mut self, url: &str) {
    let block = WebBlock::new(url);
    
    self.animator.queue(Animation::SlideUp {
        duration: 0.3,
        ease: EaseFunction::CubicOut,
    });
    
    self.blocks.push(block);
}
```

## Block Store

### Distribution Model

```yaml
# block.yaml
metadata:
  id: com.aios.blocks.spotify
  name: Spotify Player
  version: 1.2.0
  author: AIOS Team
  
requirements:
  min_canvas_version: 0.5.0
  capabilities: [audio, network]
  
assets:
  icon: assets/spotify.svg
  screenshots: [assets/screen1.png]
  
binary:
  linux_x64: bin/spotify_block.so
  linux_arm64: bin/spotify_block_arm.so
```

### Installation

```rust
// SPOC can install blocks on-demand
async fn handle_user_intent(&mut self, text: &str) {
    let intent = self.parse_intent(text);
    
    // Check if we have the right block
    if !self.has_block_for_intent(&intent) {
        // Find in store
        let block = self.block_store.search(&intent).await?;
        
        // Install it
        self.install_block(block).await?;
    }
    
    // Use the block
    self.execute_intent(intent);
}
```

## Example User Flows

### Flow: "Play my discover weekly"

```
1. User types: "Play my discover weekly"
2. SPOC recognizes: Intent::PlayMusic { playlist: "discover weekly" }
3. SPOC checks: Do we have Spotify block?
   - No: Install from block store
   - Yes: Continue
4. Create SpotifyBlock (Hybrid)
5. Authenticate via Spotify API
6. Fetch playlist
7. Start playback via GStreamer
8. Show mini player UI (native)
9. Animate in with SlideUp transition
```

### Flow: "Edit my config"

```
1. User types: "Edit my config"
2. SPOC recognizes: Intent::EditFile { file: "~/.config/..." }
3. Create CodeBlock (Native)
4. Load file into Xi-rope
5. Apply syntax highlighting via Tree-sitter
6. Show with Expand animation
7. User can now edit directly
8. On save, SPOC confirms
```

### Flow: "Show me Twitter but cleaner"

```
1. User types: "Show me Twitter but cleaner"
2. SPOC creates WebBlock
3. Loads twitter.com
4. Injects CSS to hide:
   - Sidebar
   - Trending
   - Ads
   - "What's happening"
5. Shows only timeline
6. User can still interact
7. But AI can also control
```

## Performance Considerations

### Block Rendering Priority

```rust
enum RenderPriority {
    Critical,    // Active/focused block - 60 FPS
    High,        // Visible blocks - 30 FPS
    Low,         // Background blocks - 10 FPS
    Suspended,   // Not rendered
}

// Smart rendering
impl Canvas {
    fn render_blocks(&mut self) {
        for block in &mut self.blocks {
            match block.priority() {
                Critical => block.render_full(),
                High => block.render_simplified(),
                Low => block.render_cached(),
                Suspended => {}, // Skip
            }
        }
    }
}
```

## Security Model

```rust
// Blocks run in sandboxes
struct BlockSandbox {
    capabilities: Vec<Capability>,
    filesystem: SandboxedFS,
    network: Option<NetworkAccess>,
}

enum Capability {
    FileRead(PathBuf),
    FileWrite(PathBuf),
    NetworkAccess(Vec<Domain>),
    SystemInfo,
    ProcessSpawn,
    AudioPlayback,
    CameraAccess,
}
```

## Next Steps

1. **Define core block interfaces** (Block trait)
2. **Implement first native block** (CodeBlock)
3. **Implement first web block** (WebBlock with JS injection)
4. **Implement first hybrid block** (SlackBlock)
5. **Build transition engine**
6. **Create block packaging format**
7. **Build block store backend**

---

*This is our block platform vision. Native for performance, web for compatibility, hybrid for the best of both worlds.*