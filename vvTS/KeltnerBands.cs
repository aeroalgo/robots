using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000121 RID: 289
	[HandlerCategory("vvBands&Channels"), HandlerName("KeltnerBands")]
	public class KeltnerBands : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000848 RID: 2120 RVA: 0x000233F5 File Offset: 0x000215F5
		public IList<double> Execute(ISecurity src)
		{
			return KeltnerBands.GenKeltnerBands(src, this.Context, this.AtrPeriod, this.MaPeriod, this.MaMode, this.AtrCoeff, this.BandMode);
		}

		// Token: 0x06000847 RID: 2119 RVA: 0x00023278 File Offset: 0x00021478
		public static IList<double> GenKeltnerBands(ISecurity src, IContext ctx, int _AtrPeriod, int _MaPeriod, int _MaMode, double _AtrCoeff, int _BandMode)
		{
			int count = src.get_Bars().Count;
			IList<double> list = new double[count];
			IList<double> tp = ctx.GetData("TypicalPrices", new string[]
			{
				src.get_CacheName()
			}, () => Series.TypicalPrice(src.get_Bars()));
			IList<double> data = ctx.GetData("atr", new string[]
			{
				_AtrPeriod.ToString(),
				src.get_CacheName()
			}, () => ATR.GenATR(src, _AtrPeriod, 0));
			IList<double> data2 = ctx.GetData("middle", new string[]
			{
				_MaMode.ToString(),
				_MaPeriod.ToString(),
				_MaPeriod.ToString()
			}, () => AllAverages.Gen_mMA(tp, ctx, _MaMode, _MaPeriod, _MaPeriod / 2, 1.0, 1.0));
			for (int i = 0; i < count; i++)
			{
				list[i] = data2[i] + ((_BandMode == 1) ? (data[i] * _AtrCoeff) : (-(data[i] * _AtrCoeff)));
			}
			if (_BandMode != 0)
			{
				return list;
			}
			return data2;
		}

		// Token: 0x170002A0 RID: 672
		[HandlerParameter(true, "2", Min = "1", Max = "3", Step = "1", Name = "Множитель ATR")]
		public double AtrCoeff
		{
			// Token: 0x06000843 RID: 2115 RVA: 0x000231ED File Offset: 0x000213ED
			get;
			// Token: 0x06000844 RID: 2116 RVA: 0x000231F5 File Offset: 0x000213F5
			set;
		}

		// Token: 0x1700029D RID: 669
		[HandlerParameter(true, "10", Min = "10", Max = "20", Step = "1")]
		public int AtrPeriod
		{
			// Token: 0x0600083D RID: 2109 RVA: 0x000231BA File Offset: 0x000213BA
			get;
			// Token: 0x0600083E RID: 2110 RVA: 0x000231C2 File Offset: 0x000213C2
			set;
		}

		// Token: 0x170002A1 RID: 673
		[HandlerParameter(false, "1", NotOptimized = true, Name = "BandMode:\n0-middleMA\n1-top,2-bottom")]
		public int BandMode
		{
			// Token: 0x06000845 RID: 2117 RVA: 0x000231FE File Offset: 0x000213FE
			get;
			// Token: 0x06000846 RID: 2118 RVA: 0x00023206 File Offset: 0x00021406
			set;
		}

		// Token: 0x170002A2 RID: 674
		public IContext Context
		{
			// Token: 0x06000849 RID: 2121 RVA: 0x00023421 File Offset: 0x00021621
			get;
			// Token: 0x0600084A RID: 2122 RVA: 0x00023429 File Offset: 0x00021629
			set;
		}

		// Token: 0x1700029F RID: 671
		[HandlerParameter(true, "2", Min = "0", Max = "7", Step = "1", Name = "MaMode\n0-SMA,2-EMA\n3-LWMA,5-HullMA\n6-AMA,7-LRMA")]
		public int MaMode
		{
			// Token: 0x06000841 RID: 2113 RVA: 0x000231DC File Offset: 0x000213DC
			get;
			// Token: 0x06000842 RID: 2114 RVA: 0x000231E4 File Offset: 0x000213E4
			set;
		}

		// Token: 0x1700029E RID: 670
		[HandlerParameter(true, "20", Min = "10", Max = "20", Step = "1")]
		public int MaPeriod
		{
			// Token: 0x0600083F RID: 2111 RVA: 0x000231CB File Offset: 0x000213CB
			get;
			// Token: 0x06000840 RID: 2112 RVA: 0x000231D3 File Offset: 0x000213D3
			set;
		}
	}
}
