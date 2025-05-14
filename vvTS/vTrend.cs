using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200006D RID: 109
	[HandlerCategory("vvIndicators"), HandlerName("vTrend")]
	public class vTrend : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060003DB RID: 987 RVA: 0x000151C8 File Offset: 0x000133C8
		private static IList<double> CalcVT(ISecurity _sec, IList<double> _hhv, IList<double> _llv)
		{
			double[] array = new double[_sec.get_Bars().Count];
			for (int i = 0; i < _sec.get_Bars().Count; i++)
			{
				array[i] = (_hhv[i] + _llv[i]) / 2.0;
			}
			return array;
		}

		// Token: 0x060003DA RID: 986 RVA: 0x00015150 File Offset: 0x00013350
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("vtrend", new string[]
			{
				this.Length.ToString(),
				this.Histogram.ToString(),
				sec.get_CacheName()
			}, () => vTrend.GenVTrendHisto(sec, this.Length, this.Histogram, this.Context));
		}

		// Token: 0x060003D9 RID: 985 RVA: 0x00014FB0 File Offset: 0x000131B0
		public static IList<double> GenVTrendHisto(ISecurity sec, int _length, bool _Histogram, IContext context)
		{
			IList<double> hhv = context.GetData("hhv", new string[]
			{
				_length.ToString(),
				sec.get_CacheName()
			}, () => Series.Highest(sec.get_HighPrices(), _length));
			IList<double> llv = context.GetData("llv", new string[]
			{
				_length.ToString(),
				sec.get_CacheName()
			}, () => Series.Lowest(sec.get_LowPrices(), _length));
			double[] array = new double[hhv.Count];
			array[0] = 0.0;
			IList<double> data = context.GetData("vt", new string[]
			{
				_length.ToString(),
				sec.get_CacheName()
			}, () => vTrend.CalcVT(sec, hhv, llv));
			for (int i = 1; i < hhv.Count; i++)
			{
				array[i] = array[i - 1];
				if (data[i] > data[i - 1])
				{
					array[i] = 1.0;
				}
				if (data[i] < data[i - 1])
				{
					array[i] = -1.0;
				}
			}
			if (!_Histogram)
			{
				return data;
			}
			return array;
		}

		// Token: 0x1700014D RID: 333
		public IContext Context
		{
			// Token: 0x060003DC RID: 988 RVA: 0x00015219 File Offset: 0x00013419
			get;
			// Token: 0x060003DD RID: 989 RVA: 0x00015221 File Offset: 0x00013421
			set;
		}

		// Token: 0x1700014C RID: 332
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Histogram
		{
			// Token: 0x060003D7 RID: 983 RVA: 0x00014F4D File Offset: 0x0001314D
			get;
			// Token: 0x060003D8 RID: 984 RVA: 0x00014F55 File Offset: 0x00013155
			set;
		}

		// Token: 0x1700014B RID: 331
		[HandlerParameter(true, "10", Min = "3", Max = "30", Step = "1")]
		public int Length
		{
			// Token: 0x060003D5 RID: 981 RVA: 0x00014F3C File Offset: 0x0001313C
			get;
			// Token: 0x060003D6 RID: 982 RVA: 0x00014F44 File Offset: 0x00013144
			set;
		}
	}
}
