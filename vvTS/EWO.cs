using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000025 RID: 37
	[HandlerCategory("vvIndicators"), HandlerName("Elliot Wave Oscillator")]
	public class EWO : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000158 RID: 344 RVA: 0x00006554 File Offset: 0x00004754
		public IList<double> Execute(ISecurity sec)
		{
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> arg_30_0 = sec.get_HighPrices();
			IList<double> arg_3D_0 = sec.get_LowPrices();
			IList<double> list = new List<double>(closePrices.Count);
			IList<double> mp = this.Context.GetData("MedianPrice", new string[]
			{
				sec.get_CacheName()
			}, () => Series.MedianPrice(sec.get_Bars()));
			IList<double> data = this.Context.GetData("sma", new string[]
			{
				this.Period1.ToString(),
				mp.GetHashCode().ToString()
			}, () => Series.SMA(mp, this.Period1));
			IList<double> data2 = this.Context.GetData("sma", new string[]
			{
				this.Period2.ToString(),
				mp.GetHashCode().ToString()
			}, () => Series.SMA(mp, this.Period2));
			for (int i = 0; i < closePrices.Count; i++)
			{
				double item = data[i] - data2[i];
				list.Add(item);
			}
			return list;
		}

		// Token: 0x17000073 RID: 115
		public IContext Context
		{
			// Token: 0x06000159 RID: 345 RVA: 0x000066C2 File Offset: 0x000048C2
			get;
			// Token: 0x0600015A RID: 346 RVA: 0x000066CA File Offset: 0x000048CA
			set;
		}

		// Token: 0x17000071 RID: 113
		[HandlerParameter(true, "5", Min = "5", Max = "100", Step = "5")]
		public int Period1
		{
			// Token: 0x06000154 RID: 340 RVA: 0x000064E6 File Offset: 0x000046E6
			get;
			// Token: 0x06000155 RID: 341 RVA: 0x000064EE File Offset: 0x000046EE
			set;
		}

		// Token: 0x17000072 RID: 114
		[HandlerParameter(true, "35", Min = "5", Max = "100", Step = "5")]
		public int Period2
		{
			// Token: 0x06000156 RID: 342 RVA: 0x000064F7 File Offset: 0x000046F7
			get;
			// Token: 0x06000157 RID: 343 RVA: 0x000064FF File Offset: 0x000046FF
			set;
		}
	}
}
