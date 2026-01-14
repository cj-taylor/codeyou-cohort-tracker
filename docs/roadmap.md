# Roadmap

Thinking out loud about where this could go. Take what's useful, ignore the rest.

## Quick take

If I had to pick what to tackle next:

1. **Notebooks** - gets the data mentors something to play with right away
2. **Dashboard improvements** - class selection UI, better filtering
3. **Slack bot** - mentors will actually see alerts without checking anything
4. **LMS abstraction** - not urgent, but TopHat is coming eventually
5. **Predictions** - fun ML project, needs more historical data first

Honestly though, whatever's bugging you most is probably the right thing to work on.

## Jupyter notebooks

Feels like the obvious move. The data analysis folks need something to dig into, and notebooks are perfect for exploring messy questions.

I'm picturing a `notebooks/` folder:

- `cohort_overview.ipynb` - big picture, how's everyone doing
- `student_deep_dive.ipynb` - pick one student, see their whole story
- `assignment_analysis.ipynb` - which ones are tripping people up
- `weekly_report.ipynb` - something you could actually email to someone

The API already exists. Just `requests.get()` the data and start plotting. The beauty of notebooks is how fast you can chase a hunch - "wait, what if we look at it this way?" and thirty seconds later you know.

## Reports

Sometimes you need a thing you can attach to an email. Not everyone wants to poke an API.

- CSV of student progress for the spreadsheet people
- Weekly summary PDF
- Something you can pull up in a standup without explaining what an endpoint is

## Slack or Discord

Mentors aren't going to check a dashboard every morning. But they're already in Slack.

A bot that drops in with "heads up, these 3 students haven't done anything in a week" would actually get seen. Weekly digests, completion rate updates, that kind of thing. High value, relatively low effort.

## Dashboard Improvements

Multi-class support is implemented in the backend, but the dashboard needs:

- Class selection screen on load
- Dropdown or cards to pick which class to view
- Filter all visualizations by selected class
- "Back to classes" navigation
- Show/hide archived classes toggle

The API endpoints are ready (`/classes/:class_id/...`), just needs the UI work.

## TopHat and other LMSs

OpenClass isn't forever. TopHat is on the horizon, maybe others after that.

Would be nice if the sync layer was pluggable. Same database, same API, different sources feeding it:

```text
src/
  sync/
    mod.rs          # shared traits
    openclass.rs    # what we have now
    tophat.rs       # someday
```

Each adapter handles auth and data fetching for its LMS, then maps everything to our schema.

The catch is that LMSs don't all think about things the same way. What's an "assignment" exactly? How are grades scaled? There's abstraction work to figure out. Not urgent, but try not to bake in too many OpenClass-specific assumptions.

## Flagging at-risk students

Further out, but it's a legit ML project for the data cohort.

Look at historical data: what did it look like before someone dropped off? Build a simple model (logistic regression is probably fine) to score current students against those patterns. Surface the risky ones in the dashboard or via Slack.

Real problem, real data, real outcome. Good learning project.

## Dashboard stuff

Works, but pretty basic. Eventually:

- Click a student to see their full history
- Filter by dates, assignment type, whatever
- Make it not look broken on a phone

## Skip list

Things that sound useful but probably aren't worth it:

- **Real-time sync** - data doesn't change fast enough to matter
- **User accounts** - unless this goes public, who cares
- **Microservices** - it's a cohort tracker, let's not get carried away
