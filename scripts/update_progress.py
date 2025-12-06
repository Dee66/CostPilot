#!/usr/bin/env python3
"""
CostPilot Checklist Progress Updater

Updates the progress bar in checklist.md based on completed checkbox items.
Provides a modern, visually engaging progress bar with color gradients.
"""

import re
import sys
from pathlib import Path
from typing import Tuple


# ANSI color codes for gradient effect
class Colors:
    """Color palette for progress bar - red → orange → yellow → green → cyan → blue"""
    GRADIENT = [
        '\033[38;5;196m',  # Bright Red (0-10%)
        '\033[38;5;202m',  # Orange-Red (10-20%)
        '\033[38;5;208m',  # Orange (20-30%)
        '\033[38;5;214m',  # Orange-Yellow (30-40%)
        '\033[38;5;220m',  # Yellow (40-50%)
        '\033[38;5;226m',  # Bright Yellow (50-60%)
        '\033[38;5;154m',  # Yellow-Green (60-70%)
        '\033[38;5;46m',   # Green (70-80%)
        '\033[38;5;51m',   # Cyan (80-90%)
        '\033[38;5;21m',   # Blue (90-100%)
    ]
    RESET = '\033[0m'
    BOLD = '\033[1m'


def get_color_for_percentage(percentage: float) -> str:
    """Get the appropriate color for the given percentage."""
    if percentage < 10:
        return Colors.GRADIENT[0]
    elif percentage < 20:
        return Colors.GRADIENT[1]
    elif percentage < 30:
        return Colors.GRADIENT[2]
    elif percentage < 40:
        return Colors.GRADIENT[3]
    elif percentage < 50:
        return Colors.GRADIENT[4]
    elif percentage < 60:
        return Colors.GRADIENT[5]
    elif percentage < 70:
        return Colors.GRADIENT[6]
    elif percentage < 80:
        return Colors.GRADIENT[7]
    elif percentage < 90:
        return Colors.GRADIENT[8]
    else:
        return Colors.GRADIENT[9]


def count_checkboxes(content: str) -> Tuple[int, int]:
    """
    Count total and completed checkboxes in the markdown content.
    
    Returns:
        Tuple of (completed_count, total_count)
    """
    # Match checked boxes: - [x] or - [X]
    checked_pattern = r'- \[[xX]\]'
    # Match unchecked boxes: - [ ]
    unchecked_pattern = r'- \[ \]'
    
    completed = len(re.findall(checked_pattern, content))
    unchecked = len(re.findall(unchecked_pattern, content))
    total = completed + unchecked
    
    return completed, total


def generate_progress_bar(completed: int, total: int, width: int = 98) -> str:
    """
    Generate an amazing HTML/CSS progress bar with animations and gradients.
    
    Args:
        completed: Number of completed tasks
        total: Total number of tasks
        width: Not used in HTML version, kept for API compatibility
    
    Returns:
        String representation of the HTML progress bar
    """
    if total == 0:
        percentage = 0.0
    else:
        percentage = (completed / total) * 100
    
    remaining = total - completed
    
    # Choose color scheme based on percentage
    if percentage >= 80:
        color_start = "#10b981"  # Emerald
        color_mid = "#3b82f6"    # Blue
        color_end = "#8b5cf6"    # Purple
        glow_color = "#10b981"
        emoji = "🚀"
        status = "Excellent Progress"
    elif percentage >= 60:
        color_start = "#3b82f6"  # Blue
        color_mid = "#06b6d4"    # Cyan
        color_end = "#10b981"    # Emerald
        glow_color = "#3b82f6"
        emoji = "💪"
        status = "Strong Progress"
    elif percentage >= 40:
        color_start = "#f59e0b"  # Amber
        color_mid = "#3b82f6"    # Blue
        color_end = "#06b6d4"    # Cyan
        glow_color = "#f59e0b"
        emoji = "⚡"
        status = "Building Momentum"
    elif percentage >= 20:
        color_start = "#ef4444"  # Red
        color_mid = "#f59e0b"    # Amber
        color_end = "#3b82f6"    # Blue
        glow_color = "#ef4444"
        emoji = "🔥"
        status = "Getting Started"
    else:
        color_start = "#ef4444"  # Red
        color_mid = "#f97316"    # Orange
        color_end = "#f59e0b"    # Amber
        glow_color = "#ef4444"
        emoji = "🌱"
        status = "Just Beginning"
    
def generate_progress_bar(completed: int, total: int, width: int = 98) -> str:
    """Generate a clean, functional progress bar."""
    if total == 0:
        percentage = 0.0
    else:
        percentage = (completed / total) * 100
    
    # Choose gradient colors based on progress
    if percentage >= 75:
        gradient = "linear-gradient(90deg,#2e7d32,#4caf50,#66bb6a)"
    elif percentage >= 50:
        gradient = "linear-gradient(90deg,#f59e0b,#fbbf24,#4caf50)"
    elif percentage >= 25:
        gradient = "linear-gradient(90deg,#ea580c,#f59e0b,#fbbf24)"
    else:
        gradient = "linear-gradient(90deg,#dc2626,#ea580c,#f59e0b)"
    
    html = f'''<div role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow="{int(percentage)}" style="width:94%; background:#e6eef0; border-radius:8px; padding:6px; box-shadow: inset 0 1px 2px rgba(0,0,0,0.04);">
  <div style="width:{percentage:.1f}%; background:{gradient}; color:#fff; padding:10px 12px; text-align:right; border-radius:6px; font-weight:700; transition:width 0.5s ease;">
    <span style="display:inline-block; background:rgba(0,0,0,0.12); padding:4px 8px; border-radius:999px; font-size:0.95em;">{int(percentage)}% · {completed}/{total}</span>
  </div>
</div>'''
    
    return html


def generate_stats_table(completed: int, total: int) -> str:
    """Generate additional stats section (now integrated into main progress bar)."""
    # Return empty string since stats are now in the main HTML
    return ""


def update_checklist(file_path: Path, dry_run: bool = False) -> None:
    """
    Update the progress bar in the checklist.md file.
    
    Args:
        file_path: Path to the checklist.md file
        dry_run: If True, only print the changes without writing to file
    """
    if not file_path.exists():
        print(f"❌ Error: File not found: {file_path}")
        sys.exit(1)
    
    # Read the file
    content = file_path.read_text(encoding='utf-8')
    
    # Count checkboxes
    completed, total = count_checkboxes(content)
    
    # Generate new progress bar
    progress_bar = generate_progress_bar(completed, total)
    
    # Find and replace the progress section
    # Match from after "## 📊 Overall Progress" to just before "**Target:"
    pattern = r'(## 📊 Overall Progress\n\n).*?(?=\*\*Target:)'
    
    replacement = f'\\1{progress_bar}\n\n'
    
    new_content = re.sub(pattern, replacement, content, count=1, flags=re.DOTALL)
    
    # Calculate percentage for display
    percentage = (completed / total * 100) if total > 0 else 0
    
    # Print summary with colors
    color = get_color_for_percentage(percentage)
    print(f"\n{Colors.BOLD}📊 CostPilot Progress Update{Colors.RESET}")
    print(f"{'='*50}")
    print(f"✅ Completed: {color}{completed}{Colors.RESET}/{total}")
    print(f"📈 Progress:  {color}{int(percentage)}%{Colors.RESET}")
    print(f"⏳ Remaining: {total - completed} tasks")
    print(f"{'='*50}\n")
    
    if dry_run:
        print("🔍 Dry run mode - no files will be modified")
        return
    
    # Write back to file
    file_path.write_text(new_content, encoding='utf-8')
    print(f"✨ Progress bar updated successfully in {file_path.name}")


def main():
    """Main entry point for the script."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description='Update the progress bar in CostPilot checklist.md',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog='''
Examples:
  %(prog)s                    # Update checklist.md in current directory
  %(prog)s --dry-run          # Preview changes without writing
  %(prog)s --file path/to/checklist.md  # Specify custom file path
        '''
    )
    
    parser.add_argument(
        '--file',
        type=Path,
        default=Path('checklist.md'),
        help='Path to the checklist.md file (default: checklist.md)'
    )
    
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Preview changes without modifying the file'
    )
    
    args = parser.parse_args()
    
    try:
        update_checklist(args.file, dry_run=args.dry_run)
    except KeyboardInterrupt:
        print("\n\n⚠️  Operation cancelled by user")
        sys.exit(130)
    except Exception as e:
        print(f"\n❌ Error: {e}")
        sys.exit(1)


if __name__ == '__main__':
    main()
