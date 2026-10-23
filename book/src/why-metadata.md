Why is metadata important?
==========================

Metadata is the difference between a good user experience and a bad user
experience.

On Netflix, movies are presented in an attractive fashion with artwork, titles,
actors, and other information. All content is playable on whatever device you
happen to be using, streaming starts instantly, search is useful, and a
recommendation engine surfaces movies you might like. It is usable by anyone,
of any age and with any degree of technical proficiency.

On BitTorrent, movies are presented inscrutably in text listings with weirdly
formatted titles, incorrect or missing information, and no artwork. Content may
or may not be playable on your device, streaming is impossible, search is
unstructured, and recommendations are unavailable. It is only usable by a small
and decreasing percentage of the population, and asking someone over to
"BitTorrent and chill" will be met by blinking incomprehension, followed by
derision as you struggle with your torrent client like a digital steam engine
operator, desperately trying to find seeds for the movie you want to watch.

The difference between the two systems is not due to a difference in underlying
data, every piece of content one could possibly imagine is available on
BitTorrent. It is a difference in the availability of metadata.

Netflix has a database of standardized, machine-readable metadata, underpinning
every feature of the system. BitTorrent has a disparate collection of folders
of files, with inconsistent and incomplete metadata. The user experience and
features of both systems are a direct consequence of this disparity.

As polished services driven by metadata have become more popular, the user
experience that systems based on folders of files can offer has become ever
more unacceptable and alien, regardless of cost or underlying availability of
content.

Filepack seeks to rectify this by standardizing machine-readable metadata that
can be included with folder-of-files content.

Filepack metadata is stored in a file, `metadata.filemeta`, in the root of a
filepack package as well as in the `manifest.filepack` archive, and contains
information related to what the package contains, which files in the package
contain the package content and their file formats, and who created the
package. This metadata can serve as a base for the creation of rich local and
distributed applications and services, with user experiences that compete with
and exceed those of closed centralized alternatives.
