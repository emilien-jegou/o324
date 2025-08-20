<p align="center">
  <h1 align="center">o324</h1>
</p>

<p align="center">
  <strong>CLI time-tracker for personal accountability.</strong>
  <br />
  <sup>It measures the <em>session</em>, not just the second.</sup>
</p>

**o324 is a different kind of time-tracker. Built for personal time management and self-accountability, it respects your privacy and encourage accuracy.**

Your data is yours, everything belongs to you and lives locally on your machine.

> **⚠️ Status:** This project is in active development. The core functionality is stable, but APIs may evolve. Contributions and feedback are welcome.

---

## Why another time-tracker ?

Let's say you start a Pomodoro work session at 9 AM and finish at 12 PM, you stop your timer for scheduled breaks. At the end, your log shows will show you that you worked **2h20m**.

Why? Because in a typical Pomodoro cycle, you only work ~77% of the time. The rest is spent on essential breaks—getting water, stretching, or resetting mentally. These breaks are part of the work session, they were an agreement you made when you started using this work strategy, yet you will feel penalized for them.

This can lead to bad habit:
*   **Leaving the timer running:** You want the timer to actually reflect the work-session, so you stop pausing the timer during breaks, sometimes you take no pause sometimes you do; either way your timer won't reflect it, no matter your productivity it will reflect the start and end of a session.
*   **Loss of motivation:** With inaccurate data, you can't distinguish between a high-focus session and a distracted one. Tracking becomes a chore that provides no real insight, you are looking more at a "calendar" than a time-tracking software, all of this leads to disengagement.
*   **Macro-tasking:** Since you are often leaving the timer running and you don't get into the habit of opening it task are not properly segmented, at the end of the day no matter how much work you put-in you will have worked on one thing, a good-analogy would be a "squashed" commit and no one like to code-review a squashed commit.

## The o324 Paradigm: Focus as the primary indicator

**o324** re-frames the problem by treating session quality as the primary unit of measurement. It understands that breaks are an integral part of the session, and should not be seen as a punishment.

> Instead of fragmented time entries, o324 logs your 9 AM to 12 PM block as a single **3-hour session** with a **77% activity level**.

Why is this better?

*   **It Aligns Metrics with Reality:** Your goal shifts from maximizing raw "hours worked" to optimizing your **Activity Percentage**. This single metric encourages deep work and reflects the true quality and intensity of your focus.
*   **It Reinforces Accountability:** By defining clear session boundaries, o324 keeps your work front-and-center. The act of tracking becomes intentional, leading to more accurate data and a more honest assessment of your accomplishments.


---

## Core Features

o324 runs as a system daemon, providing a robust and seamless tracking experience through a powerful CLI.

*   🧠 **Intelligent Session Grouping:** Automatically aggregates work and break periods into cohesive sessions, complete with an objective activity score.
*   🔍 **Automated Activity Detection:** Monitors system activity (e.g., window titles, idle state) to eliminate manual timer management and ensure precision.
*   🔒 **Data Integrity Engine:** Automatically stops active tasks on system sleep or shutdown (with a 3-minute grace period), preventing corrupted or abandoned timers.
*   🔔 **Proactive Nudges:** Delivers notifications when you appear to be working without an active timer, helping you maintain a consistent log.
*   ⚡ **Dynamic CLI:** A high-performance command-line interface with in-memory caching of task ID prefixes for swift, frictionless interaction (inspired by `jujutsu`).
*   ☁️ **Device Synchronization (Planned):** Sync your work sessions securely across multiple machines.

---

## Contributing

Contributions are welcome! Whether it's bug reports, feature requests, or code contributions, please feel free to open an issue or submit a pull request on our GitHub repository.

