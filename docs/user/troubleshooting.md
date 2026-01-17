# Troubleshooting Guide

This guide helps you solve common issues with Cerebras-MAKER.

## Connection Issues

### "API Key Invalid" Error

**Symptoms:**
- Error message about invalid API key
- MAKER won't start generating code

**Solutions:**
1. Double-check your API key for typos
2. Ensure you copied the entire key (no missing characters)
3. Verify your API key is active on the provider's website
4. Check if your API key has the required permissions

### "Connection Failed" Error

**Symptoms:**
- Cannot connect to AI provider
- Timeout errors

**Solutions:**
1. Check your internet connection
2. Verify the provider's service status
3. If using OpenAI-Compatible, check your Base URL
4. Try a different provider temporarily
5. Check if a firewall is blocking the connection

### Slow Response Times

**Symptoms:**
- Long delays before code generation starts
- Cockpit shows "Processing" for extended periods

**Solutions:**
1. Check your internet speed
2. Try a faster provider (Cerebras is optimized for speed)
3. Reduce the complexity of your request
4. Check if the provider is experiencing high load

## Code Generation Issues

### "Failed Code Generation" Error

**Symptoms:**
- MAKER starts but produces errors
- Generated code is incomplete

**Solutions:**
1. **Simplify your request**: Break complex tasks into smaller pieces
2. **Be more specific**: Vague requests lead to poor results
3. **Check the Cockpit**: Look for red flags or error messages
4. **Use Time Machine**: Roll back and try a different approach
5. **Verify model selection**: Ensure you're using capable models

### Generated Code Has Errors

**Symptoms:**
- Code doesn't compile or run
- Missing imports or dependencies

**Solutions:**
1. Check the Cockpit for any red flags that were raised
2. Use the Topology view to look for dependency issues
3. Ask MAKER to fix specific errors by describing them
4. Roll back with Time Machine and provide more context

### Code Doesn't Match Requirements

**Symptoms:**
- Generated code works but isn't what you wanted
- Missing features or wrong behavior

**Solutions:**
1. Answer Interrogator questions more thoroughly
2. Provide more detailed requirements
3. Include examples of expected behavior
4. Specify technologies and patterns you want used

## Time Machine / Rollback Issues

### Rollback Not Working

**Symptoms:**
- Clicking Rollback doesn't restore previous state
- Error message when trying to roll back

**Solutions:**
1. **Refresh the snapshot list**: Click the refresh button
2. **Check disk space**: Shadow Git needs space for snapshots
3. **Verify project folder**: Ensure you're in the correct workspace
4. **Check file permissions**: MAKER needs write access to the project folder

### Missing Snapshots

**Symptoms:**
- Expected snapshots aren't showing
- Timeline is empty

**Solutions:**
1. Refresh the Shadow Git view
2. Check if the project was opened correctly
3. Verify the `.shadow-git` folder exists in your project
4. Restart MAKER and reopen the project

### Squash Failed

**Symptoms:**
- Cannot combine snapshots
- Error during squash operation

**Solutions:**
1. Ensure you have at least 2 snapshots to squash
2. Check for conflicting changes between snapshots
3. Try rolling back first, then making new changes

## Graph / Visualization Issues

### Blueprint Not Loading

**Symptoms:**
- 3D view is blank or shows errors
- Nodes don't appear

**Solutions:**
1. **Wait for analysis**: Large codebases take time to analyze
2. **Check browser/WebGL**: Blueprint requires WebGL support
3. **Refresh the view**: Switch to another view and back
4. **Reduce codebase size**: Very large projects may need filtering

### Topology Graph Empty

**Symptoms:**
- No nodes or connections shown
- Graph area is blank

**Solutions:**
1. Ensure your project has been analyzed
2. Check that code files exist in the workspace
3. Wait for the initial analysis to complete
4. Try refreshing or reopening the project

### Graph Performance Issues

**Symptoms:**
- Graph is slow or laggy
- Browser becomes unresponsive

**Solutions:**
1. Close other browser tabs/applications
2. Reduce the number of visible nodes (filter by type)
3. Use a device with better graphics capabilities
4. Consider analyzing a subset of your codebase

## Performance Issues

### MAKER Running Slowly

**Symptoms:**
- Everything takes longer than expected
- UI feels sluggish

**Solutions:**
1. **Close unused applications**: Free up system resources
2. **Check RLM settings**: Lower thresholds may cause more processing
3. **Use faster models**: Some models are optimized for speed
4. **Reduce request complexity**: Simpler tasks run faster

### High Memory Usage

**Symptoms:**
- System becomes slow
- Out of memory errors

**Solutions:**
1. Restart MAKER to clear memory
2. Work with smaller codebases
3. Close the Blueprint view when not needed
4. Increase system RAM if possible

## Application Issues

### MAKER Won't Start

**Symptoms:**
- Application crashes on launch
- Nothing happens when clicking the icon

**Solutions:**
1. **Restart your computer**: Clears temporary issues
2. **Reinstall MAKER**: Download fresh from releases
3. **Check system requirements**: Ensure your system meets minimums
4. **Check for updates**: Install the latest version

### Settings Not Saving

**Symptoms:**
- API keys disappear after restart
- Configuration resets

**Solutions:**
1. Check file permissions in the app data folder
2. Run MAKER with administrator privileges (Windows)
3. Verify disk space is available
4. Try resetting settings and re-entering

### Unexpected Crashes

**Symptoms:**
- MAKER closes without warning
- Error dialogs appear

**Solutions:**
1. Note any error messages that appear
2. Check the application logs
3. Report the issue with steps to reproduce
4. Try with a fresh project to isolate the problem

## Getting More Help

If these solutions don't resolve your issue:

1. **Check the FAQ**: Common questions are answered in [faq.md](faq.md)
2. **Search Issues**: Look for similar problems on GitHub
3. **Report a Bug**: Create a new issue with:
   - Steps to reproduce
   - Expected vs actual behavior
   - System information
   - Any error messages

---

**See also**: [FAQ](faq.md) | [Getting Started](getting-started.md)

