using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000151 RID: 337
	[HandlerCategory("vvMACD"), HandlerName("ZeroLag OsMA")]
	public class ZeroLagOsMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000A8C RID: 2700 RVA: 0x0002BBE6 File Offset: 0x00029DE6
		public IList<double> Execute(IList<double> src)
		{
			return this.GenZeroLagOsMA(src, this.FastPeriod, this.SlowPeriod, this.SignalPeriod);
		}

		// Token: 0x06000A8B RID: 2699 RVA: 0x0002B960 File Offset: 0x00029B60
		public IList<double> GenZeroLagOsMA(IList<double> src, int _fastperiod, int _slowperiod, int _signalperiod)
		{
			List<double> list = new List<double>(src.Count);
			IList<double> P = src;
			double[] array = new double[P.Count];
			double[] array2 = new double[P.Count];
			IList<double> EMA1 = this.Context.GetData("ema", new string[]
			{
				_fastperiod.ToString(),
				P.GetHashCode().ToString()
			}, () => Series.EMA(P, _fastperiod));
			IList<double> data = this.Context.GetData("ema", new string[]
			{
				_fastperiod.ToString(),
				EMA1.GetHashCode().ToString()
			}, () => Series.EMA(EMA1, _fastperiod));
			IList<double> EMA3 = this.Context.GetData("ema", new string[]
			{
				_slowperiod.ToString(),
				P.GetHashCode().ToString()
			}, () => Series.EMA(P, _slowperiod));
			IList<double> data2 = this.Context.GetData("ema", new string[]
			{
				_slowperiod.ToString(),
				EMA3.GetHashCode().ToString()
			}, () => Series.EMA(EMA3, _slowperiod));
			for (int i = 0; i < P.Count; i++)
			{
				array[i] = 2.0 * EMA1[i] - data[i] - (2.0 * EMA3[i] - data2[i]);
			}
			IList<double> list2 = Series.EMA(array, _signalperiod);
			IList<double> list3 = Series.EMA(list2, _signalperiod);
			for (int j = 0; j < P.Count; j++)
			{
				array2[j] = 2.0 * list2[j] - list3[j];
			}
			for (int k = 0; k < src.Count; k++)
			{
				list.Add(array[k] - array2[k]);
			}
			return list;
		}

		// Token: 0x1700037D RID: 893
		public IContext Context
		{
			// Token: 0x06000A8D RID: 2701 RVA: 0x0002BC01 File Offset: 0x00029E01
			get;
			// Token: 0x06000A8E RID: 2702 RVA: 0x0002BC09 File Offset: 0x00029E09
			set;
		}

		// Token: 0x1700037A RID: 890
		[HandlerParameter(true, "12", Min = "5", Max = "100", Step = "5")]
		public int FastPeriod
		{
			// Token: 0x06000A85 RID: 2693 RVA: 0x0002B8D8 File Offset: 0x00029AD8
			get;
			// Token: 0x06000A86 RID: 2694 RVA: 0x0002B8E0 File Offset: 0x00029AE0
			set;
		}

		// Token: 0x1700037C RID: 892
		[HandlerParameter(true, "9", Min = "3", Max = "20", Step = "1")]
		public int SignalPeriod
		{
			// Token: 0x06000A89 RID: 2697 RVA: 0x0002B8FA File Offset: 0x00029AFA
			get;
			// Token: 0x06000A8A RID: 2698 RVA: 0x0002B902 File Offset: 0x00029B02
			set;
		}

		// Token: 0x1700037B RID: 891
		[HandlerParameter(true, "26", Min = "5", Max = "100", Step = "5")]
		public int SlowPeriod
		{
			// Token: 0x06000A87 RID: 2695 RVA: 0x0002B8E9 File Offset: 0x00029AE9
			get;
			// Token: 0x06000A88 RID: 2696 RVA: 0x0002B8F1 File Offset: 0x00029AF1
			set;
		}
	}
}
