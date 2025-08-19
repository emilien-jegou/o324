# o324

**o324 is a different kind of time-tracker. Built for personal time management and radical self-accountability, it respects your privacy and understands how real work happens—breaks and all.**

Your data is yours. Period. o324 does not collect any data. Everything belongs to you and lives locally on your machine.

> ⚠️ Project in active development.

---
## The time-tracking issue

Let's say you start a Pomodoro work session at 9 AM and finish at 12 PM. During this time, you naturally stop the timer for breaks. At noon, you look at your log and feel discouraged to see you only "worked" 2h 20m.

Why? Because in a typical Pomodoro cycle, you only work ~77% of the time. The rest is spent on essential breaks—getting water, stretching, or resetting mentally. These breaks are part of the work session, yet you will feel penalized for them.

This leads to bad habits:
*   **Leaving the timer running:** This makes your data inaccurate and defeats the purpose of tracking.
*   **Reduced self-commitment:** If the timer is always running, are you truly "on the clock"?
*   **Disengagement:** You can't quantify the productivity difference between a low-focus session and a deep-work session, you will stop tracking your work because it didn't feel rewarding.
*   **Macro-tasking:** Since you are often leaving the timer running and you don't get into the habit of opening it task are not properly segmented, at the end of the day no matter how much work you put-in you will have worked on one thing, a good-analogy would be a "squashed" commit and no one like to code-review a squashed commit.

**o324 works differently.** It automatically groups your work into **sessions**. If you worked from 9 AM to 12 PM using the Pomodoro technique, o324 logs it as a **3-hour session with a 77% activity level**.

Why this is better:
*   **Focus on Quality, Not Just Quantity:** Your goal shifts from maximizing "hours worked" to improving your **Activity Percentage**. This encourages deep work and prevents the disengagement that comes from leaving a timer running in the background.
*   **Builds a Stronger Habit:** By making you more aware of your work sessions, o324 stays top-of-mind. This leads to more accurate data and, ultimately, a more honest and rewarding feeling about the work you've accomplished.

---
## Features

o324 has to run as a daemon on your system, the CLI interact with it via D-Bus. Here are some features:
*   **Automatic Activity Detection:** Monitors your system activity (window titles) to intelligently pause your timer on inactivity.
*   **Smart Session Boundaries:** Automatically stop an active task when your machine shuts down or sleeps (3 minutes grace period), ensuring your logs are always accurate.
*   **Inactivity Notifications:** We'll wake you up if you forgot to run the timer.
*   **Device Synchronization:**
*   **Dynamic CLI interface:** In-memory caching of your task id prefixes, similar to jujutsu vcs.


---
## Contributing

Contributions are welcome! Whether it's bug reports, feature requests, or code contributions, please feel free to open an issue or submit a pull request on our GitHub repository.
