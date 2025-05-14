using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000116 RID: 278
	[HandlerCategory("vvBands&Channels"), HandlerName("Сдвиг линии по вертикали [2]")]
	public class ShiftedLine2 : IDoubleAccumHandler, ITwoSourcesHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x060007CC RID: 1996 RVA: 0x00021DF1 File Offset: 0x0001FFF1
		public IList<double> Execute(IList<double> _list1, IList<double> _list2)
		{
			return ShiftedLine2.GenShiftedLine(_list1, _list2, this.Shift);
		}

		// Token: 0x060007CB RID: 1995 RVA: 0x00021D90 File Offset: 0x0001FF90
		public static IList<double> GenShiftedLine(IList<double> _list1, IList<double> _list2, double _shift)
		{
			if (_list1.Count != _list2.Count)
			{
				return null;
			}
			double[] array = new double[_list1.Count];
			for (int i = 0; i < _list1.Count; i++)
			{
				array[i] = _list1[i] + _list2[i] + _list1[i] / 100.0 * _shift;
			}
			return array;
		}

		// Token: 0x17000274 RID: 628
		[HandlerParameter(true, "0", Min = "-10", Max = "10", Step = "0.1")]
		public double Shift
		{
			// Token: 0x060007C9 RID: 1993 RVA: 0x00021D7F File Offset: 0x0001FF7F
			get;
			// Token: 0x060007CA RID: 1994 RVA: 0x00021D87 File Offset: 0x0001FF87
			set;
		}
	}
}
