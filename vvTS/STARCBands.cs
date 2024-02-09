using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000127 RID: 295
	[HandlerCategory("vvBands&Channels"), HandlerName("STARC Bands")]
	public class STARCBands : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600088F RID: 2191 RVA: 0x00023E1C File Offset: 0x0002201C
		public IList<double> Execute(ISecurity src)
		{
			int count = src.get_Bars().Count;
			IList<double> Close = src.get_ClosePrices();
			IList<double> list = new double[count];
			IList<double> list2 = new double[count];
			IList<double> data = this.Context.GetData("ema", new string[]
			{
				this.BandsPeriod.ToString(),
				Close.GetHashCode().ToString()
			}, () => EMA.GenEMA(Close, this.BandsPeriod));
			IList<double> data2 = this.Context.GetData("atr", new string[]
			{
				this.ATRperiod.ToString(),
				src.get_CacheName()
			}, () => Series.AverageTrueRange(src.get_Bars(), this.ATRperiod));
			for (int i = this.ATRperiod; i < count; i++)
			{
				list[i] = data[i] + this.Multiplier * data2[i];
				list2[i] = data[i] - this.Multiplier * data2[i];
			}
			if (this.BandMode == 1)
			{
				return list;
			}
			if (this.BandMode == 2)
			{
				return list2;
			}
			return data;
		}

		// Token: 0x170002BA RID: 698
		[HandlerParameter(true, "15", Min = "5", Max = "20", Step = "1")]
		public int ATRperiod
		{
			// Token: 0x06000889 RID: 2185 RVA: 0x00023DAA File Offset: 0x00021FAA
			get;
			// Token: 0x0600088A RID: 2186 RVA: 0x00023DB2 File Offset: 0x00021FB2
			set;
		}

		// Token: 0x170002BC RID: 700
		[HandlerParameter(true, "1", Min = "0", Max = "2", Step = "1")]
		public int BandMode
		{
			// Token: 0x0600088D RID: 2189 RVA: 0x00023DCC File Offset: 0x00021FCC
			get;
			// Token: 0x0600088E RID: 2190 RVA: 0x00023DD4 File Offset: 0x00021FD4
			set;
		}

		// Token: 0x170002B9 RID: 697
		[HandlerParameter(true, "10", Min = "5", Max = "20", Step = "1")]
		public int BandsPeriod
		{
			// Token: 0x06000887 RID: 2183 RVA: 0x00023D99 File Offset: 0x00021F99
			get;
			// Token: 0x06000888 RID: 2184 RVA: 0x00023DA1 File Offset: 0x00021FA1
			set;
		}

		// Token: 0x170002BD RID: 701
		public IContext Context
		{
			// Token: 0x06000890 RID: 2192 RVA: 0x00023F81 File Offset: 0x00022181
			get;
			// Token: 0x06000891 RID: 2193 RVA: 0x00023F89 File Offset: 0x00022189
			set;
		}

		// Token: 0x170002BB RID: 699
		[HandlerParameter(true, "2", Min = "1", Max = "2", Step = "0.1")]
		public double Multiplier
		{
			// Token: 0x0600088B RID: 2187 RVA: 0x00023DBB File Offset: 0x00021FBB
			get;
			// Token: 0x0600088C RID: 2188 RVA: 0x00023DC3 File Offset: 0x00021FC3
			set;
		}
	}
}
