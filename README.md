# pulley-ipstore

## Problem Statement:
Imagine your team has developed a web service that receives requests from about 20 million unique IP addresses every day. You want to keep track of the IP addresses that are making the most requests to your service each day. Your job is to write a program that (1) tracks these IP addresses in memory (don’t use a database), and (2) returns the 100 most common IP addresses.

In the language of your choice, please implement these functions:

request_handled(ip_address)
This function accepts a string containing an IP address like “145.87.2.109”. This function will be called by the web service every time it handles a request. The calling code is outside the scope of this project. Since it is being called very often, this function needs to have a fast runtime.
top100()
This function should return the top 100 IP addresses by request count, with the highest traffic IP address first. This function also needs to be fast. Imagine it needs to provide a quick response (< 300ms) to display on a dashboard, even with 20 millions IP addresses. This is a very important requirement. Don’t forget to satisfy this requirement.
clear()
This function should clear all the IP addresses and tallies. In theory, it would be called at the start of each day to forget about all IP addresses and tallies.


## Thoughts
20 million is a fairly big number, especially when no database or serialization is involved. 

4 bytes/ip address * 20e6 = 80 megabytes *minimum*, and significantly more if we string-format, account for ipv6, and track how often they've visited. If we ever change our mind about storing this information and keep growing as a website, we're dangerously close to common JSON serialization size limits. 

More importantly, 20e6 is also over the 2^24 limit imposed by a number of nodejs data structures including Map and Set. I'm not sure if plain Objects also have this limit but let's be real - handling 10s of millions of anything in a single nodejs server would be approaching dangerous levels of silliness.

I want to get better at Rust so I'm going to be writing this in Rust, but any 

Not persisting this information across multiple days is an unusual design choice. Very privacy friendly but raises the question of when the cutoff point happens. 

A more interesting policy might have a rolling count - always counting the past 24 hours. That sounds very fun, more useful, but it's not what the question is asking so I'm not gonna tackle it for now.

Anyway - approach I'm taking is exactly the same as it would be for a much smaller one, just with a different language and more junk around it
* Store number of hits in a hash table (this can theoretically be split up into many components as we grow but ergonomics is much better when it's all unified)
* Keep list of top 100 ips and pointers to their frequencies
* Whenever we handle a request from an already existing IP address, see if it's in the top 100. 
  * If so, update it and re-sort list of top 100 ips.
  * If not, see if it's bigger than 100th biggest frequency. If so, swap them and re-sort the entire list of ips according to their frequencies. Insertion sort is optimal here and in the previous substep, not that it really matters.


## How to run

