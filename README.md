# scheduler kata

```console
make test lint build serve
```
```console
curl -i 0.0.0.0:3000/health
```
![almost ready](https://pbs.twimg.com/media/FYwPdWEWYAQUUlE?format=jpg&name=medium)

https://twitter.com/supabase/status/1552632972259561475/photo/1


## requirements

Expose an API that can:
* Create a task of a specific type and execution time, returning the task's ID
* Show a list of tasks, filterable by their state (whatever states you define) and/or their task type
* Show a task based on its ID
* Delete a task based on its ID
* The tasks must be persisted into some external data store (your choice).
* Process each task only once and only at/after their specified execution time.
* Support running multiple instances of your code in parallel.


## "WTF is kata"

https://en.wikipedia.org/wiki/Kata

> Kata is a Japanese word meaning "form". It refers to a detailed
> choreographed pattern of martial arts movements made to be practised alone.
> It can also be reviewed within groups and in unison when training. It is
> practised in Japanese martial arts as a way to memorize and perfect the
> movements being executed.

https://en.wikipedia.org/wiki/Kata#Outside_martial_arts

> More recently kata has come to be used in English in a more general or figurative sense, referring to any basic form, routine, or pattern of behavior that is practised to various levels of mastery.


## Howto
TBD
