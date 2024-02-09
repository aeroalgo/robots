using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200007F RID: 127
	[HandlerCategory("vvIchimoku"), HandlerName("SenkouB")]
	public class SenkouB : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600047F RID: 1151 RVA: 0x0001761C File Offset: 0x0001581C
		public IList<double> Execute(ISecurity src)
		{
			IList<double> list = new TenkanSen
			{
				Period = this.MaxPeriod,
				Context = this.Context
			}.Execute(src);
			int count = src.get_HighPrices().Count;
			double[] array = new double[count];
			for (int i = this.MaxPeriod + this.KijunPeriod; i < count; i++)
			{
				array[i] = list[i - this.KijunPeriod];
			}
			return array;
		}

		// Token: 0x17000189 RID: 393
		public IContext Context
		{
			// Token: 0x06000480 RID: 1152 RVA: 0x00017690 File Offset: 0x00015890
			get;
			// Token: 0x06000481 RID: 1153 RVA: 0x00017698 File Offset: 0x00015898
			set;
		}

		// Token: 0x17000188 RID: 392
		[HandlerParameter(true, "26", Min = "5", Max = "52", Step = "1")]
		public int KijunPeriod
		{
			// Token: 0x0600047D RID: 1149 RVA: 0x00017608 File Offset: 0x00015808
			get;
			// Token: 0x0600047E RID: 1150 RVA: 0x00017610 File Offset: 0x00015810
			set;
		}

		// Token: 0x17000187 RID: 391
		[HandlerParameter(true, "52", Min = "25", Max = "255", Step = "1")]
		public int MaxPeriod
		{
			// Token: 0x0600047B RID: 1147 RVA: 0x000175F7 File Offset: 0x000157F7
			get;
			// Token: 0x0600047C RID: 1148 RVA: 0x000175FF File Offset: 0x000157FF
			set;
		}
	}
}
