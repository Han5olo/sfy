#! /usr/bin/env python

import click
import matplotlib.pyplot as plt
from datetime import timedelta, datetime
from tabulate import tabulate
import numpy as np

from sfy.hub import Hub
from sfy.axl import AxlCollection
from sfy import signal
from sfy.timeutil import utcify
import logging

logger = logging.getLogger(__name__)


@click.group()
def axl():
    pass


@axl.command()
@click.argument('dev')
@click.option('--tx-start',
              default=None,
              help='Search in packages after this time (default: 24h ago)',
              type=click.DateTime())
@click.option('--tx-end',
              default=None,
              help='Search in packages before this time (default: now)',
              type=click.DateTime())
@click.option('--start',
              default=None,
              help='Clip results before this (default: tx-start)',
              type=click.DateTime())
@click.option('--end',
              default=None,
              help='Clip results after this (default: tx-end)',
              type=click.DateTime())
@click.option('--plot', is_flag=True, help='Plot timeseries')
def ts(dev, tx_start, tx_end, start, end, plot):
    hub = Hub.from_env()
    buoy = hub.buoy(dev)

    if tx_start is None:
        tx_start = datetime.utcnow() - timedelta(days=1)

    if tx_end is None:
        tx_end = datetime.utcnow()

    if start is None:
        start = tx_start

    if end is None:
        end = tx_end

    tx_start = utcify(tx_start)
    tx_end = utcify(tx_end)
    start = utcify(start)
    end = utcify(end)

    logger.info(f"Scanning for packages tx: {tx_start} <-> {tx_end} and clipping between {start} <-> {end}")

    pcks = buoy.axl_packages_range(tx_start, tx_end)
    logger.info(f"{len(pcks)} packages in tx range")

    pcks = AxlCollection(pcks)

    # filter packages between start and end
    pcks.clip(start, end)
    logger.info(f"{len(pcks)} in start <-> end range")

    segments = list(pcks.segments())
    logger.info(f"Collection consists of: {len(segments)} segments")

    assert len(pcks) == sum(len(s) for s in segments)

    stable = [[
        s.start,
        s.end,
        s.duration,
        timedelta(seconds = s.duration),
        s.max_gap(),
        np.nan,
        len(s),
        ] for s in segments]

    for i, _ in enumerate(stable[1:]):
        stable[i+1][5] = (stable[i+1][0] - stable[i][1])

    print(tabulate(stable, headers = ['Start', 'End', 'Duration (s)', 'Duration', 'Max Internal Gap', 'Segment Gap', 'Packages']))

    if plot:
        logger.info("Plotting..")


@axl.command(help='Plot package')
@click.argument('dev')
@click.argument('file')
def file(dev, file):
    hub = Hub.from_env()
    buoy = hub.buoy(dev)
    ax = buoy.package(file)

    a = signal.detrend(ax.z)
    _, _, w = signal.velocity(ax)
    _, _, u = signal.displacement(ax)
    u = signal.detrend(u)

    plt.figure()
    plt.title(
        f"Buoy: {buoy.dev}\n{ax.start} / {ax.received_datetime} length: {ax.duration}s f={ax.freq}Hz"
    )
    plt.plot(ax.time[:], a, label='acceleration ($m/s^2$)')
    plt.plot(ax.time[:-1], w, label='velocity ($m/s$)')
    plt.plot(ax.time[:-2], u, label='displacement ($m$)')

    print(ax.time[0])

    plt.grid()
    plt.legend()
    plt.xlabel('Time')
    plt.ylabel('Vertical movement $m$, $m/s$, $m/s^2$')

    plt.show()


@axl.command(help='Monitor buoy')
@click.argument('dev')
@click.option('--sleep',
              help='Time to sleep between update',
              default=5.0,
              type=float)
@click.option('--window', help='Time window to show', default=None, type=float)
def monitor(dev, sleep, window):
    hub = Hub.from_env()
    buoy = hub.buoy(dev)

    

    la = None
    lv = None
    lu = None

    axl = None

    while True:
        naxl = buoy.last()
        print(naxl.time[0])

        if axl is None:
            plt.ion()
            fig = plt.figure()
            ax = fig.add_subplot(111)
            plt.grid()
            plt.legend()
            plt.xlabel('Time')
            plt.ylabel('Vertical movement $m$, $m/s$, $m/s^2$')

            print("new data package")
            axl = naxl

            plt.title(
                f"Buoy: {buoy.dev}\n{axl.start} / {axl.received_datetime} length: {axl.duration}s f={axl.freq}Hz"
            )

            a = signal.detrend(axl.z)
            _, _, w = signal.velocity(axl)
            _, _, u = signal.displacement(axl)

            la, = ax.plot(axl.time[:],
                            a,
                            'k--',
                            alpha=.5,
                            label='acceleration ($m/s^2$)')
            lv, = ax.plot(axl.time[:-1],
                            w,
                            'g--',
                            alpha=.5,
                            label='velocity ($m/s$)')
            lu, = ax.plot(axl.time[:-2],
                            u,
                            'b',
                            label='displacement ($m$)')
        else:
            if (axl != naxl):
                print('Update...')
            axl = naxl
            a = signal.detrend(axl.z)
            _, _, w = signal.velocity(axl)
            _, _, u = signal.displacement(axl)

            la.set_ydata(a)
            lv.set_ydata(w)
            lu.set_ydata(u)

            la.set_xdata(axl.time[:])
            lv.set_xdata(axl.time[:-1])
            lu.set_xdata(axl.time[:-2])

            fig.canvas.draw()
            fig.canvas.flush_events()

        plt.legend()

        if window is not None:
            plt.xlim([axl.end - timedelta(seconds=window), axl.end])

        plt.pause(sleep)
