using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000080 RID: 128
	[HandlerCategory("vvIchimoku"), HandlerName("Chinkou")]
	public class Chinkou : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000485 RID: 1157 RVA: 0x000176BC File Offset: 0x000158BC
		public IList<double> Execute(ISecurity src)
		{
			IList<double> closePrices = src.get_ClosePrices();
			double[] array = new double[closePrices.Count];
			for (int i = 0; i < array.Length - this.KijunPeriod; i++)
			{
				array[i] = closePrices[i + this.KijunPeriod];
			}
			return array;
		}

		// Token: 0x1700018B RID: 395
		public IContext Context
		{
			// Token: 0x06000486 RID: 1158 RVA: 0x00017703 File Offset: 0x00015903
			get;
			// Token: 0x06000487 RID: 1159 RVA: 0x0001770B File Offset: 0x0001590B
			set;
		}

		// Token: 0x1700018A RID: 394
		[HandlerParameter(true, "26", Min = "5", Max = "52", Step = "1")]
		public int KijunPeriod
		{
			// Token: 0x06000483 RID: 1155 RVA: 0x000176A9 File Offset: 0x000158A9
			get;
			// Token: 0x06000484 RID: 1156 RVA: 0x000176B1 File Offset: 0x000158B1
			set;
		}
	}
}
