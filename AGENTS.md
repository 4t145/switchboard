# Skeleton UI Auxiliary Information

To assist AI agents in writing Skeleton UI related code, please refer to the following official documentation index file:

- **URL**: https://www.skeleton.dev/llms.txt

This file contains the documentation structure for both React and Svelte versions of Skeleton UI components. During development, if specific component usage needs to be consulted, Opencode can use the `webfetch` tool to retrieve the latest documentation index from this URL, or lookup specific component documentation based on the index.
## Styling
You should look up the "presets" sector of skeleton document to know how to use skeleton style classes, instead of use "variant-xx" classes.

# Language Guidelines

## User Communication
- **Primary Rule**: Always respond to users in the same language they used in their question/request
- If a user asks in Chinese, respond in Chinese
- If a user asks in English, respond in English
- If a user switches languages mid-conversation, adapt accordingly

## Code and Documentation
For all public-facing content that may be shared or maintained by multiple developers:

- **Code Comments**: Always use English
- **Documentation files** (README, API docs, etc.): Always use English
- **Commit messages**: Always use English
- **Variable names and function names**: Always use English
- **Error messages in code**: Always use English
- **Public interfaces and APIs**: Always use English

## Rationale
This approach ensures:
1. Better user experience through native language communication
2. International collaboration through standardized English in code/docs
3. Consistency with global development practices
4. Accessibility for international contributors 


# RUST programming

## Don't use magic value
Always reference a const value instead of hard coded value.

## Show me the error reason
Every function returns a `Result` type should have a `#Error` section in the document comments, which records the possibility of error.

## Prefer `impl Future` or `BoxFuture` then `async_trait`
Actually I don't want to use `async_trait` lib

## Never unwrap
1. You can reorganize control flow to avoid unwrap
2. If you have to call `unwrap`, use `expect` and explain why you confident to unwrap here.

## Follow rust 2024 style