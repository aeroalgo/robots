using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200014F RID: 335
	[HandlerCategory("vvMACD"), HandlerName("ZeroLag MACD")]
	public class ZeroLagMACD : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000A6C RID: 2668 RVA: 0x0002B322 File Offset: 0x00029522
		public IList<double> Execute(IList<double> src)
		{
			return this.GenZeroLagMACD(src, this.FP, this.SP, this.SignalP);
		}

		// Token: 0x06000A6B RID: 2667 RVA: 0x0002B0C4 File Offset: 0x000292C4
		public IList<double> GenZeroLagMACD(IList<double> _src, int _fastperiod, int _slowperiod, int _signalperiod)
		{
			double[] array = new double[_src.Count];
			double[] array2 = new double[_src.Count];
			IList<double> EMA1 = this.Context.GetData("ema", new string[]
			{
				_fastperiod.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.EMA(_src, _fastperiod));
			IList<double> data = this.Context.GetData("ema", new string[]
			{
				_fastperiod.ToString(),
				EMA1.GetHashCode().ToString()
			}, () => Series.EMA(EMA1, _fastperiod));
			IList<double> EMA3 = this.Context.GetData("ema", new string[]
			{
				_slowperiod.ToString(),
				_src.GetHashCode().ToString()
			}, () => Series.EMA(_src, _slowperiod));
			IList<double> data2 = this.Context.GetData("ema", new string[]
			{
				_slowperiod.ToString(),
				EMA3.GetHashCode().ToString()
			}, () => Series.EMA(EMA3, _slowperiod));
			for (int i = 0; i < _src.Count; i++)
			{
				array[i] = 2.0 * EMA1[i] - data[i] - (2.0 * EMA3[i] - data2[i]);
			}
			IList<double> list = Series.EMA(array, _signalperiod);
			IList<double> list2 = Series.EMA(list, _signalperiod);
			for (int j = 0; j < _src.Count; j++)
			{
				array2[j] = 2.0 * list[j] - list2[j];
			}
			if (!this.SignalLine)
			{
				return array;
			}
			return array2;
		}

		// Token: 0x17000370 RID: 880
		public IContext Context
		{
			// Token: 0x06000A6D RID: 2669 RVA: 0x0002B33D File Offset: 0x0002953D
			get;
			// Token: 0x06000A6E RID: 2670 RVA: 0x0002B345 File Offset: 0x00029545
			set;
		}

		// Token: 0x1700036C RID: 876
		[HandlerParameter(true, "12", Min = "5", Max = "100", Step = "5")]
		public int FP
		{
			// Token: 0x06000A63 RID: 2659 RVA: 0x0002B029 File Offset: 0x00029229
			get;
			// Token: 0x06000A64 RID: 2660 RVA: 0x0002B031 File Offset: 0x00029231
			set;
		}

		// Token: 0x1700036F RID: 879
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool SignalLine
		{
			// Token: 0x06000A69 RID: 2665 RVA: 0x0002B05C File Offset: 0x0002925C
			get;
			// Token: 0x06000A6A RID: 2666 RVA: 0x0002B064 File Offset: 0x00029264
			set;
		}

		// Token: 0x1700036E RID: 878
		[HandlerParameter(true, "9", Min = "5", Max = "50", Step = "1")]
		public int SignalP
		{
			// Token: 0x06000A67 RID: 2663 RVA: 0x0002B04B File Offset: 0x0002924B
			get;
			// Token: 0x06000A68 RID: 2664 RVA: 0x0002B053 File Offset: 0x00029253
			set;
		}

		// Token: 0x1700036D RID: 877
		[HandlerParameter(true, "26", Min = "5", Max = "100", Step = "5")]
		public int SP
		{
			// Token: 0x06000A65 RID: 2661 RVA: 0x0002B03A File Offset: 0x0002923A
			get;
			// Token: 0x06000A66 RID: 2662 RVA: 0x0002B042 File Offset: 0x00029242
			set;
		}
	}
}
