using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200007E RID: 126
	[HandlerCategory("vvIchimoku"), HandlerName("SenkouA")]
	public class SenkouA : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000477 RID: 1143 RVA: 0x00017520 File Offset: 0x00015720
		public IList<double> Execute(ISecurity src)
		{
			IList<double> list = new TenkanSen
			{
				Period = this.TenkanPeriod,
				Context = this.Context
			}.Execute(src);
			IList<double> list2 = new KijunSen
			{
				Period = this.KijunPeriod,
				Context = this.Context
			}.Execute(src);
			int count = src.get_HighPrices().Count;
			double[] array = new double[count];
			for (int i = this.KijunPeriod + this.KijunPeriod; i < count; i++)
			{
				array[i] = 0.5 * (list[i - this.KijunPeriod] + list2[i - this.KijunPeriod]);
			}
			return array;
		}

		// Token: 0x17000186 RID: 390
		public IContext Context
		{
			// Token: 0x06000478 RID: 1144 RVA: 0x000175DE File Offset: 0x000157DE
			get;
			// Token: 0x06000479 RID: 1145 RVA: 0x000175E6 File Offset: 0x000157E6
			set;
		}

		// Token: 0x17000185 RID: 389
		[HandlerParameter(true, "26", Min = "5", Max = "52", Step = "1")]
		public int KijunPeriod
		{
			// Token: 0x06000475 RID: 1141 RVA: 0x0001750F File Offset: 0x0001570F
			get;
			// Token: 0x06000476 RID: 1142 RVA: 0x00017517 File Offset: 0x00015717
			set;
		}

		// Token: 0x17000184 RID: 388
		[HandlerParameter(true, "9", Min = "3", Max = "25", Step = "1")]
		public int TenkanPeriod
		{
			// Token: 0x06000473 RID: 1139 RVA: 0x000174FE File Offset: 0x000156FE
			get;
			// Token: 0x06000474 RID: 1140 RVA: 0x00017506 File Offset: 0x00015706
			set;
		}
	}
}
