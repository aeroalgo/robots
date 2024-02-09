using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000150 RID: 336
	[HandlerCategory("vvMACD"), HandlerName("ZeroLagMACD^RSI")]
	public class zlMACD_RSI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000A81 RID: 2689 RVA: 0x0002B898 File Offset: 0x00029A98
		public IList<double> Execute(IList<double> src)
		{
			return this.GenzlMACDRSI(src, this.FastPeriod, this.SlowPeriod, this.SignalPeriod, this.RSIperiod, this.RSIcaliber);
		}

		// Token: 0x06000A80 RID: 2688 RVA: 0x0002B434 File Offset: 0x00029634
		public IList<double> GenzlMACDRSI(IList<double> src, int _fastperiod, int _slowperiod, int _signalperiod, int _rsiperiod, double _rsicaliber)
		{
			double[] array = new double[src.Count];
			IList<double> EMA1 = this.Context.GetData("ema", new string[]
			{
				_fastperiod.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.EMA(src, _fastperiod));
			IList<double> data = this.Context.GetData("ema", new string[]
			{
				_fastperiod.ToString(),
				EMA1.GetHashCode().ToString()
			}, () => Series.EMA(EMA1, _fastperiod));
			IList<double> EMA3 = this.Context.GetData("ema", new string[]
			{
				_slowperiod.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.EMA(src, _slowperiod));
			IList<double> data2 = this.Context.GetData("ema", new string[]
			{
				_slowperiod.ToString(),
				EMA3.GetHashCode().ToString()
			}, () => Series.EMA(EMA3, _slowperiod));
			for (int i = 0; i < src.Count; i++)
			{
				array[i] = 2.0 * EMA1[i] - data[i] - (2.0 * EMA3[i] - data2[i]);
			}
			double[] array2 = new double[src.Count];
			IList<double> list = Series.EMA(array, _signalperiod);
			IList<double> list2 = Series.EMA(list, _signalperiod);
			for (int j = 0; j < src.Count; j++)
			{
				array2[j] = 2.0 * list[j] - list2[j];
			}
			double num = 0.0;
			double num2 = 0.0;
			double[] array3 = new double[src.Count];
			double[] array4 = new double[src.Count];
			double[] array5 = new double[src.Count];
			for (int k = 1; k < src.Count; k++)
			{
				double num3 = array[k] - array[k - 1];
				if (num3 > 0.0)
				{
					num2 = num3;
				}
				else
				{
					num = -num3;
				}
				double num4 = (array3[k - 1] * (double)(_rsiperiod - 1) + num2) / (double)_rsiperiod;
				double num5 = (array4[k - 1] * (double)(_rsiperiod - 1) + num) / (double)_rsiperiod;
				array3[k] = num4;
				array4[k] = num5;
				if (num5 == 0.0)
				{
					array5[k] = 0.0;
				}
				else
				{
					array5[k] = _rsicaliber * (50.0 - 100.0 / (1.0 + num4 / num5));
				}
			}
			if (this.Chart)
			{
				IPane pane = this.Context.CreatePane("zlMACD^RSI", 30.0, false, false);
				pane.AddList(string.Concat(new object[]
				{
					"zlMACD(",
					_fastperiod,
					",",
					_slowperiod,
					")"
				}), array, 3, 11909109, 0, 0);
				pane.AddList("Signal(" + _signalperiod + ")", array2, 0, 13894681, 2, 0);
				pane.AddList("zlMACD^RSI(" + _rsiperiod + ")", array5, 0, 201612, 0, 0);
			}
			if (!this.SignalLine && !this.MACDLine)
			{
				return array5;
			}
			if (!this.SignalLine)
			{
				return array;
			}
			return array2;
		}

		// Token: 0x17000378 RID: 888
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Chart
		{
			// Token: 0x06000A7E RID: 2686 RVA: 0x0002B3CD File Offset: 0x000295CD
			get;
			// Token: 0x06000A7F RID: 2687 RVA: 0x0002B3D5 File Offset: 0x000295D5
			set;
		}

		// Token: 0x17000379 RID: 889
		public IContext Context
		{
			// Token: 0x06000A82 RID: 2690 RVA: 0x0002B8BF File Offset: 0x00029ABF
			get;
			// Token: 0x06000A83 RID: 2691 RVA: 0x0002B8C7 File Offset: 0x00029AC7
			set;
		}

		// Token: 0x17000371 RID: 881
		[HandlerParameter(true, "12", Min = "5", Max = "100", Step = "1")]
		public int FastPeriod
		{
			// Token: 0x06000A70 RID: 2672 RVA: 0x0002B356 File Offset: 0x00029556
			get;
			// Token: 0x06000A71 RID: 2673 RVA: 0x0002B35E File Offset: 0x0002955E
			set;
		}

		// Token: 0x17000377 RID: 887
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool MACDLine
		{
			// Token: 0x06000A7C RID: 2684 RVA: 0x0002B3BC File Offset: 0x000295BC
			get;
			// Token: 0x06000A7D RID: 2685 RVA: 0x0002B3C4 File Offset: 0x000295C4
			set;
		}

		// Token: 0x17000375 RID: 885
		[HandlerParameter(true, "16", Min = "10", Max = "50", Step = "1")]
		public double RSIcaliber
		{
			// Token: 0x06000A78 RID: 2680 RVA: 0x0002B39A File Offset: 0x0002959A
			get;
			// Token: 0x06000A79 RID: 2681 RVA: 0x0002B3A2 File Offset: 0x000295A2
			set;
		}

		// Token: 0x17000374 RID: 884
		[HandlerParameter(true, "5", Min = "5", Max = "30", Step = "1")]
		public int RSIperiod
		{
			// Token: 0x06000A76 RID: 2678 RVA: 0x0002B389 File Offset: 0x00029589
			get;
			// Token: 0x06000A77 RID: 2679 RVA: 0x0002B391 File Offset: 0x00029591
			set;
		}

		// Token: 0x17000376 RID: 886
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool SignalLine
		{
			// Token: 0x06000A7A RID: 2682 RVA: 0x0002B3AB File Offset: 0x000295AB
			get;
			// Token: 0x06000A7B RID: 2683 RVA: 0x0002B3B3 File Offset: 0x000295B3
			set;
		}

		// Token: 0x17000373 RID: 883
		[HandlerParameter(true, "9", Min = "5", Max = "50", Step = "1")]
		public int SignalPeriod
		{
			// Token: 0x06000A74 RID: 2676 RVA: 0x0002B378 File Offset: 0x00029578
			get;
			// Token: 0x06000A75 RID: 2677 RVA: 0x0002B380 File Offset: 0x00029580
			set;
		}

		// Token: 0x17000372 RID: 882
		[HandlerParameter(true, "26", Min = "5", Max = "100", Step = "1")]
		public int SlowPeriod
		{
			// Token: 0x06000A72 RID: 2674 RVA: 0x0002B367 File Offset: 0x00029567
			get;
			// Token: 0x06000A73 RID: 2675 RVA: 0x0002B36F File Offset: 0x0002956F
			set;
		}
	}
}
